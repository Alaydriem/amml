/// The cursor shape over the game window. `NONE` leaves the shape to the game.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlCursorShape(pub u8);

impl AmmlCursorShape {
    pub const NONE: Self = Self(0);
    pub const ARROW: Self = Self(1);
    pub const HAND: Self = Self(2);
    pub const TEXT: Self = Self(3);
}
