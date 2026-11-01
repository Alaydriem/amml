/// The values of the game's `ChunkState`, a one-byte enum (`ChunkState.h`).
pub struct ChunkState;

impl ChunkState {
    /// The chunk is loaded and ready.
    pub const LOADED: u8 = 11;
}
