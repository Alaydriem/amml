/// Layout mirror of the game's `SubChunk`: only its size, the stride of `LevelChunk::mSubChunks`. The
/// `SubChunk.size` entry must have this size, else the chunk reads are not offered.
#[repr(C)]
pub struct SubChunkLayout {
    _members: [u64; 13], // 0x00 mDirtyTicksCounter .. mIsInitialized
}

impl SubChunkLayout {
    /// The size of a `SubChunk`, the value of the `SubChunk.size` entry.
    pub const SIZE: usize = size_of::<SubChunkLayout>();
}

const _: () = assert!(SubChunkLayout::SIZE == 0x68);
