use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcMap;

use crate::game::CompoundTagVariantLayout;

/// Layout mirror of the game's `CompoundTag` (0x18 bytes): the vtable, then the `std::map<std::string,
/// CompoundTagVariant>` of its tags.
#[repr(C)]
pub struct CompoundTagLayout {
    _vtable: *const c_void,                      // 0x00
    pub tags: MsvcMap<CompoundTagVariantLayout>, // 0x08 mTags (head pointer, size)
}

const _: () = assert!(offset_of!(CompoundTagLayout, tags) == 0x08);
const _: () = assert!(size_of::<CompoundTagLayout>() == 0x18);
