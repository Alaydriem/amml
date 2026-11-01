use std::ffi::c_void;
use std::ptr::NonNull;

use amml_msvc::MemberCall;

use crate::game::dimension_type::DimensionType;
use crate::game::{DimensionLayout, RawObject};
use crate::{EntryId, GameEntries};

/// A `Dimension*` of the game.
pub struct Dimension<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> Dimension<'a> {
    /// Every entry the reads here use: `id` needs `Dimension.getDimensionId`; `sky_darken` needs none (the
    /// `Dimension` mirror).
    pub const ENTRIES: [EntryId; 1] = [EntryId::DIMENSION_GET_DIMENSION_ID];

    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `Dimension` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The dimension id as the game holds it. The values 0, 1 and 2 are the overworld, the nether and the end; any
    /// other value (a custom dimension) passes through unchanged. `None` when the `Dimension.getDimensionId` entry
    /// is off or its vtable slot is null. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn id(&self) -> Option<i32> {
        let slot = self.entries.get(EntryId::DIMENSION_GET_DIMENSION_ID)?;
        // SAFETY: `from_raw` guarantees a live Dimension, whose first qword is its vtable, and the entry is the
        // slot of `getDimensionId` in that vtable.
        let function = unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }?;
        // SAFETY: the slot holds `DimensionType Dimension::getDimensionId() const`. `DimensionType` is a struct of
        // one `int`, and a non-static member function returns any struct through a hidden pointer (`this` in RCX,
        // the buffer in RDX), which `MemberCall` does.
        let id: DimensionType = unsafe { MemberCall::call_sret0(function, self.ptr.as_ptr()) };
        Some(id.value)
    }

    /// The sky darkness, 0 to 15: how much the time of day dims the sky light. Always `Some`: the member is read
    /// through the `Dimension` mirror, which needs no entry. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn sky_darken(&self) -> Option<u8> {
        let dimension = self.ptr.as_ptr().cast::<DimensionLayout>();
        // SAFETY: `from_raw` guarantees a live Dimension, which `DimensionLayout` mirrors up to `mSkyDarken`.
        Some(unsafe { (&raw const (*dimension).sky_darken).read_volatile() })
    }
}
