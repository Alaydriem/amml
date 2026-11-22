use std::mem::offset_of;

use crate::BlockTessellatorCacheLayout;

/// Mirror of the game's `BlockTessellator` (479E0h bytes on 26.63 and 70.26: `mov edx,479E0h` before the constructor
/// call, survey section 7, row M2; the member layout is the same on both builds, row M1). The constructor zeroes the word
/// at 0 (the normals flag at 1), stores the region at 8, zeroes the dword at 80h (the render layer, which
/// `tessellateInWorld` compares) and the byte at E8h (ambient occlusion), and stores the word 100h at F0h, so F1h (the
/// cached calculation flag) starts at 1; `tessellateInWorld` writes its fifth argument to F1h. `BlockTessellator.mBlockCache`
/// (6B0h) is the anchor entry. The loader allocates it on the game heap and writes through the game's pointer only.
#[repr(C)]
pub struct BlockTessellatorLayout {
    _flag0: u8,                                   // 0x000
    pub use_normals: bool,                        // 0x001
    _head: [u8; 6],                               // 0x002
    pub region: usize,                            // 0x008 mRegion
    _to_layer: [u8; 0x80 - 0x10],                 // 0x010
    pub rendering_layer: i32,                     // 0x080 mRenderingLayer
    _to_occlusion: [u8; 0xE8 - 0x84],             // 0x084
    pub apply_ambient_occlusion: bool,            // 0x0E8
    _to_cached: [u8; 0xF1 - 0xE9],                // 0x0E9
    pub use_cached_calculation: bool,             // 0x0F1
    _to_cache: [u8; 0x6B0 - 0xF2],                // 0x0F2
    pub block_cache: BlockTessellatorCacheLayout, // 0x6B0 mBlockCache
    _tail: [u8; 0x479E0 - 0x6B0 - 0x426A0],       // 0x42D50
}

impl BlockTessellatorLayout {
    /// The size the survey read from the exe on 26.63 and 70.26.
    pub const SIZE: usize = 0x479E0;
    /// The offset of `mBlockCache`, the value the `BlockTessellator.mBlockCache` entry has for this mirror.
    pub const BLOCK_CACHE: u64 = offset_of!(BlockTessellatorLayout, block_cache) as u64;
}

const _: () = assert!(offset_of!(BlockTessellatorLayout, use_normals) == 0x1);
const _: () = assert!(offset_of!(BlockTessellatorLayout, region) == 0x8);
const _: () = assert!(offset_of!(BlockTessellatorLayout, rendering_layer) == 0x80);
const _: () = assert!(offset_of!(BlockTessellatorLayout, apply_ambient_occlusion) == 0xE8);
const _: () = assert!(offset_of!(BlockTessellatorLayout, use_cached_calculation) == 0xF1);
const _: () = assert!(offset_of!(BlockTessellatorLayout, block_cache) == 0x6B0);
const _: () = assert!(size_of::<BlockTessellatorLayout>() == BlockTessellatorLayout::SIZE);
