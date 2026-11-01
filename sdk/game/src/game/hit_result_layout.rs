use std::mem::offset_of;

use crate::game::BlockPos;

/// Layout mirror of the start of the game's `HitResult` (136 bytes, 0x88), up to and with `mPos`. It is a prefix: use it
/// only through a pointer to a game `HitResult`, never by value.
#[repr(C)]
pub struct HitResultLayout {
    _start_pos: [f32; 3], // 0x00 mStartPos
    _ray_dir: [f32; 3],   // 0x0C mRayDir
    pub kind: i32,        // 0x18 mType (HitResultType: Tile 0, Entity 1, EntityOutOfRange 2, NoHit 3)
    pub facing: u8,       // 0x1C mFacing
    pub block: BlockPos,  // 0x20 mBlock
    pub pos: [f32; 3],    // 0x2C mPos
}

const _: () = assert!(offset_of!(HitResultLayout, kind) == 0x18);
const _: () = assert!(offset_of!(HitResultLayout, facing) == 0x1C);
const _: () = assert!(offset_of!(HitResultLayout, block) == 0x20);
const _: () = assert!(offset_of!(HitResultLayout, pos) == 0x2C);
const _: () = assert!(size_of::<HitResultLayout>() == 0x38);
