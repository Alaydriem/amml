/// Who caused an event.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlOrigin(pub u8);

impl AmmlOrigin {
    pub const GAME: Self = Self(0);
    pub const THIS_MOD: Self = Self(1);
    pub const OTHER_MOD: Self = Self(2);
}
