/// A face of a block, as the game numbers it.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid; a value above
/// `EAST` is a face this version does not name.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlFace(pub u8);

impl AmmlFace {
    pub const DOWN: Self = Self(0);
    pub const UP: Self = Self(1);
    pub const NORTH: Self = Self(2);
    pub const SOUTH: Self = Self(3);
    pub const WEST: Self = Self(4);
    pub const EAST: Self = Self(5);
}
