/// The offsets in a `LevelChunk` that the chunk reads use, of one of the two layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkOffsets {
    /// `mMin`: three ints, y second.
    pub min: usize,
    /// `mMax`: three ints, y second.
    pub max: usize,
    /// `mLoadState`: one byte.
    pub load_state: usize,
    /// `mSubChunks`: the begin, end and capacity pointers.
    pub sub_chunks: usize,
    /// `mHeightmap`: 256 shorts.
    pub heightmap: usize,
    /// `mRenderHeightmap`: 256 shorts.
    pub render_heightmap: usize,
}
