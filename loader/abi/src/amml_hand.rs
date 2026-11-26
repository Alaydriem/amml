/// A hand of the player.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid; the loader gives
/// `INVALID_ARGUMENT` for a hand it does not know.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlHand(pub u8);

impl AmmlHand {
    pub const MAIN: Self = Self(0);
    pub const OFF: Self = Self(1);
}
