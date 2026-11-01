/// Layout mirror of the game's `SubChunk`: only its size, the stride of `LevelChunk::mSubChunks`. The
/// `SubChunk.size` entry must have this size, else the chunk readers refuse the sub-chunks.
#[repr(C)]
pub struct SubChunkLayout {
    _members: [u64; 13], // 0x00 mDirtyTicksCounter .. mIsInitialized
}

const _: () = assert!(size_of::<SubChunkLayout>() == 0x68);
