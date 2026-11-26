/// How the loader clips the cursor for a mod.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlCursorClip(pub u8);

impl AmmlCursorClip {
    /// No clip: the mod's request is dropped.
    pub const NONE: Self = Self(0);
    /// A 2 x 2 box at the centre of the client area.
    pub const CENTRE: Self = Self(1);
}
