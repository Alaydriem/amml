use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcString;

use crate::game::{HashedStringLayout, MaterialFlagsLayout};

/// Layout mirror of the game's `BlockType` when it holds its Material by value (26.60), up to and with the full
/// name. Only the Material differs from [`crate::game::BlockTypePointerLayout`]; the full name is at E0h on both
/// builds. [`crate::MaterialLayout::Value`] selects it.
#[repr(C)]
pub struct BlockTypeValueLayout {
    _vtable: *const c_void,            // 0x00
    _description_id: MsvcString,       // 0x08 mDescriptionId
    pub material: MaterialFlagsLayout, // 0x28 mMaterial, by value
    _material_padding: [u8; 6],        // 0x2A
    _components: [u64; 12],            // 0x30 mComponents
    _raw_name: HashedStringLayout,     // 0x90 mNameInfo.mRawName
    _namespace_name: MsvcString,       // 0xC0 mNameInfo.mNamespaceName
    pub full_name: HashedStringLayout, // 0xE0 mNameInfo.mFullName
}

const _: () = assert!(offset_of!(BlockTypeValueLayout, material) == 0x28);
const _: () = assert!(offset_of!(BlockTypeValueLayout, full_name) == 0xE0);
