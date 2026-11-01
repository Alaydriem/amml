use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcVec;

use crate::game::SubChunkLayout;

/// Layout mirror of the game's `LevelChunk` on 26.60, up to and with `mRenderHeightmap`. The members up to
/// `mLoadState` are at their 26.51 places; `mSubChunks` and every member after it are 88h later (the 26.60 derive
/// data: `mSubChunks` 1C8h, `mHeightmap` C08h, `mRenderHeightmap` E08h).
///
/// The `LevelChunk.mSubChunks` entry selects the mirror: this one is for a build whose `mSubChunks` is at
/// [`LevelChunkLayout2660::SUB_CHUNKS`].
#[repr(C)]
pub struct LevelChunkLayout2660 {
    _lock_and_level: [u8; 0x58],             // 0x000 mBlockEntityAccessLock, mLevel
    pub dimension: *mut c_void,              // 0x058 mDimension
    pub min: [i32; 3],                       // 0x060 mMin
    pub max: [i32; 3],                       // 0x06C mMax
    pub position: [i32; 2],                  // 0x078 mPosition
    _flags_to_actor_links: [u8; 0x70],       // 0x080 mLightingFixupDone .. mUnresolvedActorLinks
    pub load_state: u8,                      // 0x0F0 mLoadState
    _state_to_sub_chunks: [u8; 0xD7],        // 0x0F1 mIsCurrentLoadStateIndeterminate .. (88h more than 26.51)
    pub sub_chunks: MsvcVec<SubChunkLayout>, // 0x1C8 mSubChunks
    _biomes_and_cached_data: [u8; 0xA28],    // 0x1E0 mBiomes, mCachedData
    pub heightmap: [i16; 256],               // 0xC08 mHeightmap
    pub render_heightmap: [i16; 256],        // 0xE08 mRenderHeightmap
}

impl LevelChunkLayout2660 {
    /// The offset of `mSubChunks`, the value the `LevelChunk.mSubChunks` entry has for this layout.
    pub const SUB_CHUNKS: u64 = offset_of!(LevelChunkLayout2660, sub_chunks) as u64;
}

const _: () = assert!(offset_of!(LevelChunkLayout2660, dimension) == 0x58);
const _: () = assert!(offset_of!(LevelChunkLayout2660, min) == 0x60);
const _: () = assert!(offset_of!(LevelChunkLayout2660, max) == 0x6C);
const _: () = assert!(offset_of!(LevelChunkLayout2660, position) == 0x78);
const _: () = assert!(offset_of!(LevelChunkLayout2660, load_state) == 0xF0);
const _: () = assert!(offset_of!(LevelChunkLayout2660, sub_chunks) == 0x1C8);
const _: () = assert!(offset_of!(LevelChunkLayout2660, heightmap) == 0xC08);
const _: () = assert!(offset_of!(LevelChunkLayout2660, render_heightmap) == 0xE08);
