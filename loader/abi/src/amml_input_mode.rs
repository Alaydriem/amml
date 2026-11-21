/// The input mode of a screen view: the device the game takes the pointer from.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. The values
/// `MOUSE` to `MOTION_CONTROLLER` are the game's own; any other value is `UNKNOWN`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlInputMode(pub u8);

impl AmmlInputMode {
    pub const UNKNOWN: Self = Self(0);
    pub const MOUSE: Self = Self(1);
    pub const TOUCH: Self = Self(2);
    pub const GAME_PAD: Self = Self(3);
    pub const MOTION_CONTROLLER: Self = Self(4);
}
