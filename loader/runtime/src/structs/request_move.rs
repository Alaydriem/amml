/// One move of a request, between the slots of the player inventory (0 to 35).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestMove {
    /// The stacks of two slots change places; both slots hold a stack.
    Swap { a: u8, b: u8 },
    /// `count` items of `from` go to `to`, which is empty or holds a stack of the same item.
    Place { from: u8, to: u8, count: u8 },
    /// `count` items of `from` are thrown out.
    Drop { from: u8, count: u8 },
    /// The pick of a creative item. The kind has its place, the loader does not run it yet.
    Pick,
}
