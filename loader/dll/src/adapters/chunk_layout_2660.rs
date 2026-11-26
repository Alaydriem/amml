use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of the game's `LevelChunk` on 70.26, up to and with `mRenderHeightmap`; the `LevelChunk.mSubChunks`
/// value 228h selects it. The members up to `mLoadState` are at the offsets of `ChunkLayout2651`; `mSubChunks` and
/// every member after it, the heightmaps among them, are 60h later. Read from the 70.26 exe: the function that
/// updates the light of a chunk column reads `mMin` at 60h, `mLoadState` at F0h, `mDimension` at 58h, `mHeightmap` at
/// C68h and `mRenderHeightmap` at E68h.
#[repr(C)]
pub struct ChunkLayout2660 {
    _lock_and_level: [u8; 0x58],          // 0x000 mBlockEntityAccessLock, mLevel
    pub dimension: *mut c_void,           // 0x058 mDimension
    pub min: [i32; 3],                    // 0x060 mMin
    pub max: [i32; 3],                    // 0x06C mMax
    pub position: [i32; 2],               // 0x078 mPosition
    _flags_to_actor_links: [u8; 0x70],    // 0x080 mLightingFixupDone .. mUnresolvedActorLinks
    pub load_state: u8,                   // 0x0F0 mLoadState
    _state_to_sub_chunks: [u8; 0x137],    // 0x0F1 mIsCurrentLoadStateIndeterminate .. before mSubChunks
    pub sub_chunks: [usize; 3],           // 0x228 mSubChunks: begin, end, capacity
    _biomes_and_cached_data: [u8; 0xA28], // 0x240 mBiomes, mCachedData
    pub heightmap: [i16; 256],            // 0xC68 mHeightmap
    pub render_heightmap: [i16; 256],     // 0xE68 mRenderHeightmap
}

impl ChunkLayout2660 {
    /// The offset of `mSubChunks`, the value the `LevelChunk.mSubChunks` entry has for this layout.
    pub const SUB_CHUNKS: u64 = offset_of!(ChunkLayout2660, sub_chunks) as u64;
}

const _: () = assert!(offset_of!(ChunkLayout2660, dimension) == 0x58);
const _: () = assert!(offset_of!(ChunkLayout2660, min) == 0x60);
const _: () = assert!(offset_of!(ChunkLayout2660, max) == 0x6C);
const _: () = assert!(offset_of!(ChunkLayout2660, position) == 0x78);
const _: () = assert!(offset_of!(ChunkLayout2660, load_state) == 0xF0);
const _: () = assert!(offset_of!(ChunkLayout2660, sub_chunks) == 0x228);
const _: () = assert!(offset_of!(ChunkLayout2660, heightmap) == 0xC68);
const _: () = assert!(offset_of!(ChunkLayout2660, render_heightmap) == 0xE68);
