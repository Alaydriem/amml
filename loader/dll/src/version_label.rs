use std::ffi::c_void;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_msvc::{MsvcHeap, MsvcString};

use crate::{Boundary, LoaderState};

/// `std::string MinecraftScreenModel::getVersionString() const`: rcx the model, rdx the hidden result.
type GetVersionStringFn = unsafe extern "C" fn(*const c_void, *mut MsvcString) -> *mut MsvcString;

static ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static LABEL: OnceLock<VersionLabel> = OnceLock::new();

/// The game's version text, as the start, pause and settings screens show it, replaced with
/// `AMML / <instance folder>`, so each running game names its instance. The game's label truncates at
/// about 26 characters, so the text is kept short; the instance folder name carries the game version.
///
/// The loader hooks `MinecraftScreenModel::getVersionString`, lets the game build its own text, and
/// swaps the result for the label. Without the entry or the game heap the game text stays as it is.
pub struct VersionLabel {
    instance: String,
}

impl VersionLabel {
    pub const ENTRY_ID: &'static str = "MinecraftScreenModel.getVersionString";

    pub fn new(instance: &str) -> Self {
        Self { instance: instance.to_string() }
    }

    /// The instance name: the name of the game folder.
    pub fn instance_name(game: &Path) -> String {
        game.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| "unknown".to_string())
    }

    /// The label that replaces the game's version text.
    pub fn text(&self) -> String {
        format!("AMML / {}", self.instance)
    }

    /// Hooks the version text. A missing entry, a missing game heap or a refused hook is logged and
    /// leaves the game's text unchanged.
    pub fn install(state: &'static LoaderState, label: VersionLabel) {
        if !state.heap().is_available() {
            curia::info!("version label unchanged: the game heap is not available");
            return;
        }
        let Ok(target) = state.resolve(Self::ENTRY_ID) else {
            curia::info!("version label unchanged: the entry did not resolve", { entry: Self::ENTRY_ID });
            return;
        };
        if LABEL.set(label).is_err() {
            return;
        }
        let detour = Self::detour as GetVersionStringFn as *const c_void;
        // SAFETY: the entry is the start of `getVersionString`, a .pdata function with a full prologue;
        // `detour` has its signature; `ORIGINAL` is a static `AtomicPtr`, read only through the atomic.
        let installed = unsafe {
            state.install_loader_hook(target as *const c_void, detour, 0, ORIGINAL.as_ptr().cast::<*const c_void>())
        };
        match installed {
            Ok(_) => curia::info!("version label installed", { entry: Self::ENTRY_ID }),
            Err(status) => curia::warn!("version label unchanged: the hook was refused", { status: status.0 }),
        }
    }

    unsafe extern "C" fn detour(model: *const c_void, out: *mut MsvcString) -> *mut MsvcString {
        let original = ORIGINAL.load(Ordering::Acquire);
        // SAFETY: the engine set `ORIGINAL` to the next hook or the trampoline before it moved the thunk
        // to this detour; both have the signature of `getVersionString`.
        let original = unsafe { std::mem::transmute::<*mut c_void, GetVersionStringFn>(original) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(model, out) };
        // SAFETY: `out` is the string the original just built on the game heap.
        Boundary::run((), || unsafe { Self::rewrite(out) });
        result
    }

    /// Replaces the game's text in `out` with the label. On any failure `out` keeps the game's text.
    ///
    /// # Safety
    /// `out` is null or a live `std::string` whose heap block, if any, came from the game allocator.
    unsafe fn rewrite(out: *mut MsvcString) {
        let Some(label) = LABEL.get() else {
            return;
        };
        if out.is_null() {
            return;
        }
        // SAFETY: the caller guarantees `out` is a live string; nothing else holds it during the call.
        let string = unsafe { &mut *out };
        let text = label.text();
        let heap = MsvcHeap::new(Self::heap_new, Self::heap_delete);
        // SAFETY: the two functions call the game `operator new` and `operator delete`.
        let Ok(replacement) = (unsafe { MsvcString::new_in(&heap, &text) }) else {
            return;
        };
        // SAFETY: the old block came from the game allocator, which `heap` calls, and is not used again.
        unsafe { string.release(&heap) };
        *string = replacement;
    }

    extern "C" fn heap_new(size: usize) -> *mut u8 {
        LoaderState::get().map_or(std::ptr::null_mut(), |state| state.heap().alloc(size))
    }

    extern "C" fn heap_delete(ptr: *mut u8, size: usize) {
        if let Some(state) = LoaderState::get() {
            // SAFETY: `MsvcHeap` passes back a block this heap allocated, with its size.
            unsafe { state.heap().free(ptr, size) };
        }
    }
}
