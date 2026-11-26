/// The kinds of real input a lease blocks, as bits.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlInputBlocks(pub u32);

impl AmmlInputBlocks {
    pub const KEYS: Self = Self(1);
    pub const MOUSE_BUTTONS: Self = Self(2);
    pub const LOOK: Self = Self(4);
    pub const MOVE: Self = Self(8);
    pub const GAMEPAD: Self = Self(16);
    pub const FULL: Self = Self(31);
}
