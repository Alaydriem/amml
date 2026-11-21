/// What a `CaptureRing` slot release tells the caller about the slot's buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotRelease {
    /// The slot is Free in an open ring: keep its buffer, the render thread writes into it again.
    Returned,
    /// The ring is closed: free the slot's buffer now.
    FreeNow,
    /// The ring holds no such slot for that present; nothing changed.
    NotHeld,
}
