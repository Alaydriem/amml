use amml_runtime::{CreativeEntry, RequestSlot, RequestStep};

/// What the request adapter reads of the game and the acts it does in it, so the adapter's order of work (read, decide
/// under the lock, act with no lock) is the same for the game and for a test.
pub trait RequestPort {
    /// Whether a screen is open. With `strict` only the HUD is no screen; otherwise a toast over it is none either.
    fn screen_open(&mut self, strict: bool) -> bool;

    /// The 36 slots of the local player's inventory; `None` when they cannot be read.
    fn read_slots(&mut self) -> Option<Vec<Option<RequestSlot>>>;

    /// Whether the local player is in the creative game mode.
    fn is_creative(&mut self) -> bool;

    /// The creative entry of the item the block state at `block` makes; `None` for a block with no creative entry.
    fn creative_entry(&mut self, block: usize) -> Option<CreativeEntry>;

    /// Does one act of a request. False when the game could not do it.
    fn run(&mut self, step: &RequestStep) -> bool;
}
