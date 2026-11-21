use std::mem::offset_of;

/// The game's `HitResult` (88h bytes), the result `BlockSource::clip` writes through its hidden result pointer.
///
/// Read from the exe on 26.63 and 70.26: the no-hit path of `clip` (0x32da050 / 0x47ab6f0) and of the block box clip
/// (0x3822390 / 0x47b9b00) write the kind 3 at 18h, the facing 0 at 1Ch, the block at 20h, the end point at 2Ch, clear
/// the weak entity reference at 38h..48h (the entity id -1 at 48h) and clear up to the qword at 7Dh; the tile hit path
/// of the block box clip writes the kind 0 at 18h, the facing (0 to 5) at 1Ch, the block position at 20h and the point
/// at 2Ch. A caller of `clip` (0x32db4b0 / 0x47acd10) destroys the result by giving back one weak count of the control
/// block at 40h (`lock dec [rcx+0Ch]`, then slot 1 at zero).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct HitResultLayout {
    pub start: [f32; 3],      // 0x00 mStartPos
    pub direction: [f32; 3],  // 0x0C mRayDir
    pub kind: i32,            // 0x18 mType (HitResultType: 0 tile, 3 no hit)
    pub facing: u8,           // 0x1C mFacing
    _pad_1d: [u8; 3],         // 0x1D
    pub block: [i32; 3],      // 0x20 mBlock
    pub pos: [f32; 3],        // 0x2C mPos
    pub weak_object: u64,     // 0x38 mEntity (WeakEntityRef) storage
    pub weak_control: u64,    // 0x40 mEntity control block
    pub entity_id: u32,       // 0x48 mEntity id
    _pad_4c: u32,             // 0x4C
    pub entity_box: [f32; 6], // 0x50 mEntityAABB
    pub is_hit_liquid: u8,    // 0x68 mIsHitLiquid
    pub liquid_facing: u8,    // 0x69 mLiquidFacing
    _pad_6a: [u8; 2],         // 0x6A
    pub liquid: [i32; 3],     // 0x6C mLiquid
    pub liquid_pos: [f32; 3], // 0x78 mLiquidPos
    pub indirect_hit: u8,     // 0x84 mIndirectHit
    _pad_85: [u8; 3],         // 0x85
}

impl HitResultLayout {
    /// `HitResultType::Tile`.
    pub const TILE: i32 = 0;
}

const _: () = assert!(offset_of!(HitResultLayout, kind) == 0x18);
const _: () = assert!(offset_of!(HitResultLayout, block) == 0x20);
const _: () = assert!(offset_of!(HitResultLayout, pos) == 0x2C);
const _: () = assert!(offset_of!(HitResultLayout, weak_control) == 0x40);
const _: () = assert!(offset_of!(HitResultLayout, entity_box) == 0x50);
const _: () = assert!(offset_of!(HitResultLayout, indirect_hit) == 0x84);
const _: () = assert!(size_of::<HitResultLayout>() == 0x88);
