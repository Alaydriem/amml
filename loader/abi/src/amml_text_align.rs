/// How a text sits in the rectangle of a `UiDraw::text` call.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlTextAlign(pub u8);

impl AmmlTextAlign {
    /// The text starts at the left edge of the rectangle.
    pub const LEFT: Self = Self(0);
    /// The text is centered in the rectangle.
    pub const CENTER: Self = Self(1);
    /// The text ends at the right edge of the rectangle.
    pub const RIGHT: Self = Self(2);
}
