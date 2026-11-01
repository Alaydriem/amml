use std::mem::offset_of;

/// Layout mirror of the game's heap `Material` of 26.51: the type, then one bool byte per flag. Solid when
/// `solid` is not 0.
#[repr(C)]
pub struct MaterialBoolsLayout {
    pub material_type: u8,        // 0 mType
    pub never_buildable: u8,      // 1 mNeverBuildable
    pub liquid: u8,               // 2 mLiquid
    pub blocks_motion: u8,        // 3 mBlocksMotion
    pub blocks_precipitation: u8, // 4 mBlocksPrecipitation
    pub solid: u8,                // 5 mSolid
    pub super_hot: u8,            // 6 mSuperHot
}

const _: () = assert!(offset_of!(MaterialBoolsLayout, solid) == 5);
