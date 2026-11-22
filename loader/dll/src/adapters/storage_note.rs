/// A storage event seen off the game thread, kept for the next `FrameUpdate` (After).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageNote {
    /// The storage was opened; the folder path copied inside the constructor detour.
    Opened(String),
    /// The storage finished its shutdown.
    Closed,
}
