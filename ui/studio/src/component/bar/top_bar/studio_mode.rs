/// The studio's mode. It follows playback (task 14d): playing is Preview, paused is Edit. The top bar tints in Preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudioMode {
    /// The free camera; keys and scenes can change.
    Edit,
    /// The scene camera drives; edits are locked.
    Preview,
}
