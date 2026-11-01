use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_data::{EntryId, EntryKind};
use amml_runtime::EntryTable;

/// The game `operator new`. It is `C-unwind` because it throws a C++ exception on out-of-memory;
/// the exception unwinds into the `extern "C"` `heap_new` of the API table, which ends the process.
pub type GameNewFn = unsafe extern "C-unwind" fn(usize) -> *mut u8;
/// The game `operator delete`. The unsized overload ignores the second argument on x64.
pub type GameDeleteFn = unsafe extern "C" fn(*mut u8, usize);

/// Allocates and frees on the game heap by calling the game `operator new` and `operator delete`.
///
/// The two entries resolve as a pair: when either is missing, `alloc` returns null and `free` does
/// nothing, and one warning is logged. Memory a mod hands to the game must come from here, never
/// from Rust or from another allocator.
///
/// The game `operator new` throws on out-of-memory. A C++ exception must not cross `extern "C"`, so
/// out-of-memory inside `alloc` aborts the process at the `heap_new` boundary, in a defined way. A
/// `nothrow` overload would avoid that and can replace the entry when one is recipe-derived.
///
/// The round trip (`check`) allocates and frees one block and logs the result, once per heap. A host
/// that cannot call game code at start-up defers it (`defer_check`) to the first `alloc`.
pub struct GameHeap {
    functions: Option<(GameNewFn, GameDeleteFn)>,
    warned: AtomicBool,
    deferred: AtomicBool,
    checked: Once,
}

impl GameHeap {
    pub const NEW_ID: &'static str = "core.operator_new";
    pub const DELETE_ID: &'static str = "core.operator_delete";
    const CHECK_SIZE: usize = 64;

    /// A heap over two functions. With either one missing the heap is unavailable.
    pub fn new(new: Option<GameNewFn>, delete: Option<GameDeleteFn>) -> Self {
        Self {
            functions: new.zip(delete),
            warned: AtomicBool::new(false),
            deferred: AtomicBool::new(false),
            checked: Once::new(),
        }
    }

    /// A heap over the resolved `core.operator_new` and `core.operator_delete`.
    ///
    /// # Safety
    /// The two entries, when present, are `function` entries of the game module at `module_base`, so
    /// that `module_base + rva` is the code of `operator new` and `operator delete`.
    pub unsafe fn from_table(table: &EntryTable, module_base: u64) -> Self {
        let new = Self::address(table, Self::NEW_ID, module_base).map(|address| {
            // SAFETY: the caller guarantees the entry is the game `operator new`.
            unsafe { std::mem::transmute::<usize, GameNewFn>(address) }
        });
        let delete = Self::address(table, Self::DELETE_ID, module_base).map(|address| {
            // SAFETY: the caller guarantees the entry is the game `operator delete`.
            unsafe { std::mem::transmute::<usize, GameDeleteFn>(address) }
        });
        Self::new(new, delete)
    }

    pub fn is_available(&self) -> bool {
        self.functions.is_some()
    }

    /// Allocates and frees one block, so a wrong `operator new` or `operator delete` entry shows
    /// before a mod depends on it, and logs `game heap ok` or the failure. Runs once per heap; later
    /// calls do nothing.
    pub fn check(&self) {
        self.checked.call_once(|| self.round_trip());
    }

    /// Runs `check` on the first `alloc` instead of now: the host cannot call game code yet.
    pub fn defer_check(&self) {
        // Relaxed: the host defers before any mod is loaded, and a mod reaches `alloc` only through
        // the loader state, whose publication orders this store before the load.
        self.deferred.store(true, Ordering::Relaxed);
    }

    /// Allocates `size` bytes on the game heap; null when the heap is unavailable. The first call
    /// runs a deferred `check` first.
    pub fn alloc(&self, size: usize) -> *mut u8 {
        if self.deferred.load(Ordering::Relaxed) {
            self.check();
        }
        self.raw_alloc(size)
    }

    fn raw_alloc(&self, size: usize) -> *mut u8 {
        match self.functions {
            // SAFETY: `new` is the game `operator new`, which takes any size.
            Some((new, _)) => unsafe { new(size) },
            None => {
                self.warn_once();
                std::ptr::null_mut()
            }
        }
    }

    /// Frees a block from `alloc` that was `size` bytes; does nothing when the heap is unavailable.
    ///
    /// # Safety
    /// `ptr` must come from `alloc` of this heap with the same `size`, and must not be freed twice.
    pub unsafe fn free(&self, ptr: *mut u8, size: usize) {
        match self.functions {
            // SAFETY: the caller guarantees `ptr` and `size`; `delete` is the game `operator delete`.
            Some((_, delete)) => unsafe { delete(ptr, size) },
            None => self.warn_once(),
        }
    }

    fn round_trip(&self) {
        if !self.is_available() {
            curia::warn!("game heap round trip skipped: the heap is unavailable");
            return;
        }
        let block = self.raw_alloc(Self::CHECK_SIZE);
        if block.is_null() {
            curia::error!("game heap returned null", { size: Self::CHECK_SIZE });
            return;
        }
        // SAFETY: `block` came from `raw_alloc(CHECK_SIZE)` of this heap just above and is freed once.
        unsafe { self.free(block, Self::CHECK_SIZE) };
        curia::info!("game heap ok", { size: Self::CHECK_SIZE });
    }

    fn address(table: &EntryTable, id: &str, module_base: u64) -> Option<usize> {
        let resolved = table.get(&EntryId(id.to_string()))?;
        if resolved.kind != EntryKind::Function {
            return None;
        }
        usize::try_from(module_base.checked_add(resolved.value)?).ok()
    }

    fn warn_once(&self) {
        if !self.warned.swap(true, Ordering::Relaxed) {
            curia::warn!("the game heap is unavailable: operator new and operator delete did not both resolve", {
                new: Self::NEW_ID,
                delete: Self::DELETE_ID
            });
        }
    }
}
