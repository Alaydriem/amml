use crate::SlotRef;

/// The one action of one item stack request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestAction {
    /// Exchanges the stacks of `from` and `to`; both slots hold a stack.
    Swap { from: SlotRef, to: SlotRef },
    /// Moves `count` items of `from` into the empty slot `to`; its net id is written as 0.
    Place { count: u8, from: SlotRef, to: SlotRef },
    /// Moves `count` items of `from` onto the stack in `to`, named with its net id.
    PlaceOnto { count: u8, from: SlotRef, to: SlotRef },
    /// Throws `count` items of `from` out, not randomly.
    Drop { count: u8, from: SlotRef },
    /// Makes `count` items of the creative item `creative_net_id` and takes them into the slot `to`; one request of two
    /// actions. With `replacing` (the held count and the held slot with its net id) the held stack is destroyed first:
    /// three actions.
    CraftCreative { creative_net_id: u32, count: u8, to: SlotRef, replacing: Option<(u8, SlotRef)> },
}
