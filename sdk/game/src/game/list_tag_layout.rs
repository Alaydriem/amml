use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcVec;

/// Layout mirror of the first 0x20 bytes of the game's `ListTag`: the vtable, then the vector of owning pointers to
/// the element tags.
#[repr(C)]
pub struct ListTagLayout {
    _vtable: *const c_void,          // 0x00
    pub items: MsvcVec<*mut c_void>, // 0x08 mList (std::vector<std::unique_ptr<Tag>>: begin, end, capacity end)
}

const _: () = assert!(offset_of!(ListTagLayout, items) == 0x08);
const _: () = assert!(size_of::<ListTagLayout>() == 0x20);
