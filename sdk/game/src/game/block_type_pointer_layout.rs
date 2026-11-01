use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcString;

use crate::game::{HashedStringLayout, MaterialBoolsLayout};

/// Layout mirror of the game's `BlockType` when it holds its `Material const&` as a pointer (26.51), up to and
/// with the full name. [`crate::MaterialLayout::Pointer`] selects it.
#[repr(C)]
pub struct BlockTypePointerLayout {
    _vtable: *const c_void,                   // 0x00
    _description_id: MsvcString,              // 0x08 mDescriptionId
    pub material: *const MaterialBoolsLayout, // 0x28 mMaterial
    _components: [u64; 12],                   // 0x30 mComponents
    _raw_name: HashedStringLayout,            // 0x90 mNameInfo.mRawName
    _namespace_name: MsvcString,              // 0xC0 mNameInfo.mNamespaceName
    pub full_name: HashedStringLayout,        // 0xE0 mNameInfo.mFullName
}

const _: () = assert!(offset_of!(BlockTypePointerLayout, material) == 0x28);
const _: () = assert!(offset_of!(BlockTypePointerLayout, full_name) == 0xE0);
