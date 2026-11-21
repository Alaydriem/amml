/// One move of `request_moves`, between the slots of the player inventory (0 to 35: 0 to 8 the hotbar, 9 to 35 the
/// main inventory). The meaning of `a`, `b` and `count` depends on `kind`:
///
/// - `SWAP`: the stacks of slot `a` and slot `b` change places; `count` is 0.
/// - `PLACE`: `count` items of slot `a` go to slot `b`; `b` is empty, or holds a stack of the same item.
/// - `DROP`: `count` items of slot `a` are thrown out; `b` is 0.
///
/// A plain integer for `kind`, so a value from the other side of the boundary can never be invalid.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlInventoryMove {
    pub kind: u8,
    pub a: u8,
    pub b: u8,
    pub count: u8,
}

impl AmmlInventoryMove {
    pub const SWAP: u8 = 1;
    pub const PLACE: u8 = 2;
    pub const DROP: u8 = 3;
    /// Reserved for the pick of a creative item; the loader gives `NOT_AVAILABLE` for it.
    pub const PICK: u8 = 4;
}

const _: () = assert!(core::mem::size_of::<AmmlInventoryMove>() == 4);
