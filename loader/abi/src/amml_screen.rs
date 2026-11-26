/// The screen on top of the client.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. Area
/// `client.top_screen` version 2 added `HUD`, `LOADING`, `TOAST` and `CONTAINER`; a reader built for version 1
/// reads them as `OTHER`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlScreen(pub u8);

impl AmmlScreen {
    pub const OTHER: Self = Self(0);
    pub const START: Self = Self(1);
    pub const PAUSE: Self = Self(2);
    /// The in-world HUD with no menu open.
    pub const HUD: Self = Self(3);
    /// A loading, saving or other progress screen.
    pub const LOADING: Self = Self(4);
    /// A notice drawn over the screen under it.
    pub const TOAST: Self = Self(5);
    /// A screen that shows a container (the inventory, a chest, a furnace and the like).
    pub const CONTAINER: Self = Self(6);
}
