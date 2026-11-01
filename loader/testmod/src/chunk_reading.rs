/// What the probe read from the chunk of the player's column.
pub struct ChunkReading {
    /// The chunk position (x, z) in chunks.
    pub pos: Option<[i32; 2]>,
    /// The lowest block y of the chunk.
    pub min_y: Option<i32>,
    /// The highest block y of the chunk.
    pub max_y: Option<i32>,
    /// Whether the chunk is fully loaded.
    pub loaded: bool,
    /// The number of sub-chunks.
    pub sub_chunks: usize,
    /// The heightmap index of the player's column.
    pub index: usize,
    /// The heightmap value at `index`.
    pub height: Option<i16>,
}

impl ChunkReading {
    /// Whether the block y `y` is inside a chunk from `min_y` up to `max_y`, which is inclusive. A
    /// `max_y` that could not be read gives `false`.
    pub fn contains_y(min_y: i32, max_y: Option<i32>, y: i32) -> bool {
        max_y.is_some_and(|max_y| min_y <= y && y <= max_y)
    }
}
