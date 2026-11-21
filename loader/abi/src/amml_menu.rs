/// A menu page the loader can open (`ClientOpenMenuFn`).
///
/// A plain integer so that a value from the other side of the boundary can never be invalid; an unknown value
/// gives `INVALID_ARGUMENT`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlMenu(pub u32);

impl AmmlMenu {
    /// The Servers tab of the Play screen.
    pub const SERVERS: Self = Self(1);
}
