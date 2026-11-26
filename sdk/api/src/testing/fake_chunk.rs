use crate::{ChunkLoad, ColumnSample};

/// One chunk of a `FakeWorld`: how much of it the client has, and the sample of each of its 256 columns (entry
/// `z * 16 + x`). `LevelChunk::read_columns` copies the samples of the columns asked for when the chunk is
/// `ChunkLoad::Loaded`, and marks them `ColumnSample::SKIPPED` otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct FakeChunk {
    pub load: ChunkLoad,
    pub samples: [ColumnSample; 256],
}

impl FakeChunk {
    /// A chunk in state `load` whose columns are all `ColumnSample::default()`.
    pub fn new(load: ChunkLoad) -> Self {
        Self { load, samples: [ColumnSample::default(); 256] }
    }
}
