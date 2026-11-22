use std::mem::offset_of;

/// Mirror of the game's `BlockTessellatorCache` (426A0h bytes, the same on 26.63 and 70.26, survey section 7, row M3):
/// the 20 x 20 x 20 window of blocks `BlockTessellator::tessellateInWorld` reads while its cached calculation is on. Cell
/// (x, y, z) of the window is at index (x - mPos.x) * 400 + (y - mPos.y) * 20 + (z - mPos.z) (row M4). `getBlock` puts
/// the default state of `minecraft:air` at +0 when it is null and gives +0 for a position outside the window; for a cell
/// that is null or holds the `minecraft:info_update` placeholder it reads `mRegion` (the world) and keeps the result; the
/// extra block getter does the same for the extra blocks. `setRegion` sets every block and extra block to that
/// placeholder, every light value to FFFFh (not computed) and every graphics pointer to null (filled from `getForBlock`
/// on the first read), on both builds; `getLightColor` takes a light value up to FEFFh as computed and reads the region
/// for a higher one. The loader writes through the game's pointer only.
#[repr(C)]
pub struct BlockTessellatorCacheLayout {
    pub air_block: usize,            // 0x00000 the air block (minecraft:air), set lazily by getBlock
    pub pos: [i32; 3],               // 0x00008 mPos, the window corner
    _pad: [u8; 4],                   // 0x00014
    pub region: usize,               // 0x00018 mRegion
    pub light_colors: [u16; 8000],   // 0x00020 the light of each cell
    pub blocks: [usize; 8000],       // 0x03EA0 the block of each cell
    pub extra_blocks: [usize; 8000], // 0x138A0 the extra block of each cell
    pub block_graphics: [[usize; 8000]; 2], // 0x232A0 the graphics of each cell, per layer
}

impl BlockTessellatorCacheLayout {
    /// The size the survey read from the exe on 26.63 and 70.26.
    pub const SIZE: usize = 0x426A0;
}

const _: () = assert!(offset_of!(BlockTessellatorCacheLayout, pos) == 0x8);
const _: () = assert!(offset_of!(BlockTessellatorCacheLayout, region) == 0x18);
const _: () = assert!(offset_of!(BlockTessellatorCacheLayout, light_colors) == 0x20);
const _: () = assert!(offset_of!(BlockTessellatorCacheLayout, blocks) == 0x3EA0);
const _: () = assert!(offset_of!(BlockTessellatorCacheLayout, extra_blocks) == 0x138A0);
const _: () = assert!(offset_of!(BlockTessellatorCacheLayout, block_graphics) == 0x232A0);
const _: () = assert!(size_of::<BlockTessellatorCacheLayout>() == BlockTessellatorCacheLayout::SIZE);
