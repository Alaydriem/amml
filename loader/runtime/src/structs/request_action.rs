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
}
