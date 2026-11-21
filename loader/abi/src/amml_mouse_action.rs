/// What a mouse event reports.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlMouseAction(pub u8);

impl AmmlMouseAction {
    pub const MOVE: Self = Self(0);
    pub const BUTTON: Self = Self(1);
    pub const WHEEL: Self = Self(2);
}
