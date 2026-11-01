use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::RawObject;
use crate::{EntryId, GameEntries};

/// A `SubChunk*` of the game.
pub struct SubChunk<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> SubChunk<'a> {
    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `SubChunk` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// Whether the sub-chunk is still the client-request placeholder (the client has not received its blocks).
    /// `None` when the entry is off. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn is_placeholder(&self) -> Option<bool> {
        let function = usize::try_from(self.entries.get(EntryId::SUB_CHUNK_IS_PLACE_HOLDER_SUB_CHUNK)?).ok()?;
        // SAFETY: the entry is the address of `bool SubChunk::isPlaceHolderSubChunk() const` and `from_raw`
        // guarantees a live SubChunk.
        Some(unsafe { RawObject::call_bool(function as *const c_void, self.ptr.as_ptr()) })
    }
}
