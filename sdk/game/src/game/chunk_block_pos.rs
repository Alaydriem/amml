/// A `ChunkBlockPos` of the game: the position of a block inside a chunk column.
///
/// `x` and `z` are 0 to 15. `y` is relative to the chunk minimum height, so it is not the world y.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ChunkBlockPos {
    pub x: u8,
    pub z: u8,
    pub y: i16,
}
