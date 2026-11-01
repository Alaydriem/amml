use std::mem::offset_of;

/// Layout mirror of the `Material` that `BlockType` holds by value on 26.60: the type, then the 26.51 bools as
/// bits of one byte in their 26.51 order (1 neverBuildable, 2 liquid, 4 blocksMotion, 8 blocksPrecipitation, 10h
/// solid, 20h superHot).
#[repr(C)]
pub struct MaterialFlagsLayout {
    pub material_type: u8, // 0 mType
    pub flags: u8,         // 1 the flag bits
}

impl MaterialFlagsLayout {
    /// The solid bit of `flags` (26.60: `test byte ptr [material+1],10h` at the `getMaterial` call sites).
    pub const SOLID: u8 = 0x10;
}

const _: () = assert!(offset_of!(MaterialFlagsLayout, flags) == 1);
