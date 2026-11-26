use std::mem::offset_of;

use crate::MeshDataLayout;

/// Mirror of the start of the game's `Tessellator`, up to `mTessellating` (survey section 8.2, the same on 26.63 and
/// 70.26): the mesh data a build fills, the vertex count `end` reads, and the flag `begin` and `end` pair on. A prefix:
/// the loader only reads and writes through the game's pointer. `Tessellator.mTessellating` is the anchor entry.
#[repr(C)]
pub struct TessellatorLayout {
    _head: [u8; 8],                // 0x000
    pub mesh_data: MeshDataLayout, // 0x008 mMeshData
    _between: [u8; 0x298 - 0x168], // 0x168
    pub count: u32,                // 0x298 mCount
    _pad: [u8; 4],                 // 0x29C
    pub tessellating: bool,        // 0x2A0 mTessellating
}

impl TessellatorLayout {
    /// The offset of `mTessellating`, the value the `Tessellator.mTessellating` entry has for this mirror.
    pub const TESSELLATING: u64 = offset_of!(TessellatorLayout, tessellating) as u64;
}

const _: () = assert!(offset_of!(TessellatorLayout, mesh_data) == 8);
const _: () = assert!(offset_of!(TessellatorLayout, count) == 0x298);
const _: () = assert!(offset_of!(TessellatorLayout, tessellating) == 0x2A0);
