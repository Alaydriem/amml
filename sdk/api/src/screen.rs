use amml_abi::AmmlScreen;

/// The screen on top of the client. A screen this crate does not know is `Other`.
///
/// `Hud`, `Loading`, `Toast` and `Container` come from area `client.top_screen` version 2
/// (`Area::CLIENT_TOP_SCREEN`); a loader that offers only version 1 gives `Other` for them.
#[non_exhaustive]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Other = 0,
    /// The title screen, or a menu page while no world is joined.
    Start = 1,
    Pause = 2,
    /// The in-world HUD with no menu open.
    Hud = 3,
    /// A loading, saving or other progress screen.
    Loading = 4,
    /// A notice drawn over the screen under it. `top_screen` reports it; the screen under it is not known.
    Toast = 5,
    /// A screen that shows a container (the inventory, a chest, a furnace and the like).
    Container = 6,
}

impl From<AmmlScreen> for Screen {
    fn from(screen: AmmlScreen) -> Self {
        match screen {
            AmmlScreen::START => Self::Start,
            AmmlScreen::PAUSE => Self::Pause,
            AmmlScreen::HUD => Self::Hud,
            AmmlScreen::LOADING => Self::Loading,
            AmmlScreen::TOAST => Self::Toast,
            AmmlScreen::CONTAINER => Self::Container,
            _ => Self::Other,
        }
    }
}
