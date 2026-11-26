/// A mouse button.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlMouseButton(pub u8);

impl AmmlMouseButton {
    pub const LEFT: Self = Self(0);
    pub const RIGHT: Self = Self(1);
    pub const MIDDLE: Self = Self(2);
    pub const X1: Self = Self(3);
    pub const X2: Self = Self(4);
}
