/// What a `WindowChanged` event reports.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlWindowChange(pub u8);

impl AmmlWindowChange {
    /// The DPI of the window changed.
    pub const DPI: Self = Self(0);
    /// The display mode or resolution changed.
    pub const DISPLAY: Self = Self(1);
    /// The window moved.
    pub const MOVED: Self = Self(2);
}
