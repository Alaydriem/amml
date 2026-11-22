/// One move of a request, between the slots of the player inventory (0 to 35).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestMove {
    /// The stacks of two slots change places; both slots hold a stack.
    Swap { a: u8, b: u8 },
    /// `count` items of `from` go to `to`, which is empty or holds a stack of the same item.
    Place { from: u8, to: u8, count: u8 },
    /// `count` items of `from` are thrown out.
    Drop { from: u8, count: u8 },
    /// A full stack of the creative item `creative_net_id` made into the empty hotbar slot `slot` (0 to 8); `count`
    /// items.
    Pick { slot: u8, creative_net_id: u32, count: u8 },
}
