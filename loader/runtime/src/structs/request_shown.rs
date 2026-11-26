/// What the client shows for a move it predicts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestShown {
    /// The two slots and the net id of the stack moved into the second.
    Swap { a: u8, b: u8, net_id: i32 },
    /// A stack of a creative item made in the hotbar slot.
    Pick { slot: u8, creative_net_id: u32, count: u8 },
}
