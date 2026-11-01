use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcVec;

use crate::game::SubChunkLayout;

/// Layout mirror of the game's `LevelChunk` on 26.51, up to and with `mRenderHeightmap`.
///
/// The `LevelChunk.mSubChunks` entry selects the mirror: this one is for a build whose `mSubChunks` is at
/// [`LevelChunkLayout2651::SUB_CHUNKS`].
#[repr(C)]
pub struct LevelChunkLayout2651 {
    _lock_and_level: [u8; 0x58],             // 0x000 mBlockEntityAccessLock, mLevel
    pub dimension: *mut c_void,              // 0x058 mDimension
    pub min: [i32; 3],                       // 0x060 mMin
    pub max: [i32; 3],                       // 0x06C mMax
    pub position: [i32; 2],                  // 0x078 mPosition
    _flags_to_actor_links: [u8; 0x70],       // 0x080 mLightingFixupDone .. mUnresolvedActorLinks
    pub load_state: u8,                      // 0x0F0 mLoadState
    _state_to_tick_queues: [u8; 0x4F],       // 0x0F1 mIsCurrentLoadStateIndeterminate .. mRandomTickQueue
    pub sub_chunks: MsvcVec<SubChunkLayout>, // 0x140 mSubChunks
    _biomes_and_cached_data: [u8; 0xA28],    // 0x158 mBiomes, mCachedData
    pub heightmap: [i16; 256],               // 0xB80 mHeightmap
    pub render_heightmap: [i16; 256],        // 0xD80 mRenderHeightmap
}

impl LevelChunkLayout2651 {
    /// The offset of `mSubChunks`, the value the `LevelChunk.mSubChunks` entry has for this layout.
    pub const SUB_CHUNKS: u64 = offset_of!(LevelChunkLayout2651, sub_chunks) as u64;
}

const _: () = assert!(offset_of!(LevelChunkLayout2651, dimension) == 0x58);
const _: () = assert!(offset_of!(LevelChunkLayout2651, min) == 0x60);
const _: () = assert!(offset_of!(LevelChunkLayout2651, max) == 0x6C);
const _: () = assert!(offset_of!(LevelChunkLayout2651, position) == 0x78);
const _: () = assert!(offset_of!(LevelChunkLayout2651, load_state) == 0xF0);
const _: () = assert!(offset_of!(LevelChunkLayout2651, sub_chunks) == 0x140);
const _: () = assert!(offset_of!(LevelChunkLayout2651, heightmap) == 0xB80);
const _: () = assert!(offset_of!(LevelChunkLayout2651, render_heightmap) == 0xD80);
