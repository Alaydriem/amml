use amml_msvc::{MsvcHeap, MsvcVec};
use amml_runtime::EmitterEntry;

use crate::LoaderState;

type NewFn = unsafe extern "C" fn(usize) -> *mut u8;
type DeleteFn = unsafe extern "C" fn(*mut u8, usize);

/// Adds one entry to the game's thread-local light emitter vector (`std::vector` of 16-byte entries at
/// `LightPropagation.tlsEmitterVector` in the game's TLS block, `gs:[58h][_tls_index]`). A full vector
/// grows as the MSVC STL grows it, through the game heap.
pub struct EmitterPush {
    tls_index: usize,
    offset: usize,
    heap: MsvcHeap,
}

impl EmitterPush {
    const TLS_INDEX: &'static str = "core.tls_index";
    const VECTOR: &'static str = "LightPropagation.tlsEmitterVector";
    const NEW: &'static str = "core.operator_new";
    const DELETE: &'static str = "core.operator_delete";

    /// The push over its four entries; `None` when one is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let new = value(Self::NEW).filter(|address| *address != 0)?;
        let delete = value(Self::DELETE).filter(|address| *address != 0)?;
        // SAFETY: the two entries are the game's `operator new` and sized `operator delete`.
        let heap = unsafe {
            MsvcHeap::new(std::mem::transmute::<usize, NewFn>(new), std::mem::transmute::<usize, DeleteFn>(delete))
        };
        let tls_index = value(Self::TLS_INDEX).filter(|address| *address != 0)?;
        Some(Self { tls_index, offset: value(Self::VECTOR)?, heap })
    }

    /// Appends `entry` to this thread's emitter vector; false, and nothing changed, when the vector looks
    /// wrong or the heap gives nothing.
    ///
    /// # Safety
    /// Called inside the game's light spread core on this thread, which owns the vector during the call.
    pub unsafe fn push(&self, entry: &[u8; EmitterEntry::SIZE]) -> bool {
        // SAFETY: as the caller guarantees.
        let Some(vector) = (unsafe { self.vector() }) else { return false };
        // SAFETY: a live vector of 16-byte trivially copyable entries the game frees with its heap; no other
        // thread uses it during the call.
        unsafe { (*vector).push_in(&self.heap, *entry).is_ok() }
    }

    // This thread's emitter vector.
    //
    // Safety: the game's TLS block of this thread is set up (true inside the light spread core).
    unsafe fn vector(&self) -> Option<*mut MsvcVec<[u8; EmitterEntry::SIZE]>> {
        let tls_array: *const usize;
        // SAFETY: gs:[58h] is the TEB ThreadLocalStoragePointer on x64 Windows.
        unsafe {
            std::arch::asm!("mov {}, gs:[0x58]", out(reg) tls_array, options(nostack, readonly, preserves_flags))
        };
        if tls_array.is_null() {
            return None;
        }
        // SAFETY: the game's `_tls_index` global (u32), at a derived address in the image.
        let index = usize::try_from(unsafe { (self.tls_index as *const u32).read() }).ok()?;
        // SAFETY: the TLS array holds one block pointer per module index.
        let block = unsafe { tls_array.add(index).read() };
        if block == 0 {
            return None;
        }
        Some(block.checked_add(self.offset)? as *mut MsvcVec<[u8; EmitterEntry::SIZE]>)
    }
}
