/// The kind of a screen the game draws, as far as the loader tells screens apart.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid; a reader
/// treats a value it does not know as `OTHER`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlScreenKind(pub u8);

impl AmmlScreenKind {
    pub const OTHER: Self = Self(0);
    pub const SMALL_CHEST: Self = Self(1);
    pub const LARGE_CHEST: Self = Self(2);
    pub const BARREL: Self = Self(3);
    pub const SHULKER_BOX: Self = Self(4);
    pub const ENDER_CHEST: Self = Self(5);
    pub const HOPPER: Self = Self(6);
    pub const DISPENSER: Self = Self(7);
    pub const DROPPER: Self = Self(8);
    pub const PLAYER_INVENTORY: Self = Self(9);
}
