use amml_runtime::{RequestSlot, RequestStep};

/// What the request adapter reads of the game and the acts it does in it, so the adapter's order of work (read, decide
/// under the lock, act with no lock) is the same for the game and for a test.
pub trait RequestPort {
    /// Whether a screen is open. With `strict` only the HUD is no screen; otherwise a toast over it is none either.
    fn screen_open(&mut self, strict: bool) -> bool;

    /// The 36 slots of the local player's inventory; `None` when they cannot be read.
    fn read_slots(&mut self) -> Option<Vec<Option<RequestSlot>>>;

    /// Does one act of a request. False when the game could not do it.
    fn run(&mut self, step: &RequestStep) -> bool;
}
