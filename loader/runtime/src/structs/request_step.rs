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
}
