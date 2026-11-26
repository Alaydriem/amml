use amml_abi::FrameEnd;

/// Typed reads of a `FrameEnd`: the game is about to end a frame. Raised Before `MinecraftGame::endFrame`, held game
/// clock or not, on the game thread.
pub trait FrameEndExt {
    /// The loader's count of `endFrame` calls, counted while a mod listens to `FrameEnd` or holds the game clock.
    fn count(&self) -> u64;
}

impl FrameEndExt for FrameEnd {
    fn count(&self) -> u64 {
        self.count
    }
}
