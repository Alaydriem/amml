use std::ffi::c_void;
use std::ptr::NonNull;

use amml_msvc::MemberCall;

use crate::game::{BlockPos, Brightness, LevelChunk, RawObject};
use crate::{EntryId, GameEntries};

/// A `BlockSource*` of the game.
pub struct BlockSource<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> BlockSource<'a> {
    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `BlockSource` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The chunk at the chunk position (`x`, `z`), through the `getChunk(int, int)` virtual. `None` when the entry
    /// is off, the vtable slot is null or the game returns null (the chunk is not loaded). Game thread; the read
    /// is not thread-checked.
    #[allow(unsafe_code)]
    pub fn chunk(&self, x: i32, z: i32) -> Option<LevelChunk<'a>> {
        let slot = self.entries.get(EntryId::BLOCK_SOURCE_GET_CHUNK)?;
        // SAFETY: `from_raw` guarantees a live BlockSource, whose first qword is its vtable, and the entry is the
        // slot of `getChunk(int, int)` in that vtable.
        let function = unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }?;
        // SAFETY: the slot holds `LevelChunk* BlockSource::getChunk(int, int) const`.
        let call: unsafe extern "C" fn(*mut c_void, i32, i32) -> *mut c_void = unsafe { std::mem::transmute(function) };
        // SAFETY: the signature is as above and `this` is a live BlockSource.
        let chunk = unsafe { call(self.ptr.as_ptr(), x, z) };
        // SAFETY: the game returns a live LevelChunk, or null.
        unsafe { LevelChunk::from_raw(chunk, self.entries) }
    }

    /// The sky light and block light at `pos`, through `getBrightnessPair`. `None` when the entry is off or its
    /// address is 0. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn brightness(&self, pos: &BlockPos) -> Option<Brightness> {
        let function = usize::try_from(self.entries.get(EntryId::BLOCK_SOURCE_GET_BRIGHTNESS_PAIR)?).ok()?;
        if function == 0 {
            return None;
        }
        // SAFETY: the entry is the address of `BrightnessPair BlockSource::getBrightnessPair(BlockPos const&)
        // const`, a non-static member that returns a struct through a hidden pointer (`this` in RCX, the buffer in
        // RDX, the position in R8), which `MemberCall` does. `Brightness` is the 2-byte `#[repr(C)]` mirror of the
        // pair, `from_raw` guarantees a live BlockSource and `pos` lives across the call.
        Some(unsafe {
            MemberCall::call_sret1::<Brightness, *const BlockPos>(
                function as *const c_void,
                self.ptr.as_ptr(),
                std::ptr::from_ref(pos),
            )
        })
    }
}
