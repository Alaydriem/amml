/// A client state a mod reads or sets.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlClientFlag(pub u8);

impl AmmlClientFlag {
    pub const WORLD_ACTIVE: Self = Self(0);
    pub const SHOWING_MENU: Self = Self(1);
    pub const MOUSE_CAPTURED: Self = Self(2);
    pub const HIDE_HUD: Self = Self(3);
}
