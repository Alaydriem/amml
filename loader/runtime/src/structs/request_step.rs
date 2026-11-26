/// One act the loader does for a request, in order. The book decides; the loader does it, outside the book's lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestStep {
    /// Sends `Interact(OpenInventory)` for the local player.
    SendInteract,
    /// Sends one `ItemStackRequest` packet made from these payload bytes.
    SendRequests(Vec<u8>),
    /// Sends the client's `ContainerClose` (type none) for the container id.
    SendClose(i8),
    /// Swaps the stacks of two player slots on the client, as the game's own slot swap does.
    Predict { a: u8, b: u8 },
    /// Swaps them back, only while slot `b` still holds the stack with `net_id` that the predict moved there.
    Unpredict { a: u8, b: u8, net_id: i32 },
    /// Shows `count` items of the creative item `creative_net_id` in the empty hotbar slot `slot` on the client.
    PredictPick { slot: u8, creative_net_id: u32, count: u8 },
    /// Takes the predicted stack out of `slot` again: the stack the pick replaced goes back (or the slot is empty again
    /// when it held none), only while the slot still holds the predicted stack (a stack with no net id).
    UnpredictPick { slot: u8 },
    /// An accepted pick named no net id for `slot`: the stack keeps the net id 0 until the game names it. The loader logs
    /// it, so a wrong offset in the answer shows in the log.
    NetIdMissing { slot: u8 },
    /// Gives the stack in `slot` the net id the server named for it, only while it still has none.
    AdoptNetId { slot: u8, net_id: i32 },
}
