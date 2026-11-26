use amml_abi::AmmlWindowChange;

/// What a `WindowChanged` reports. A value this crate does not know is `Other`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowChange {
    /// The DPI of the window changed.
    Dpi,
    /// The display mode or resolution changed.
    Display,
    /// The window moved.
    Moved,
    Other,
}

impl From<AmmlWindowChange> for WindowChange {
    fn from(change: AmmlWindowChange) -> Self {
        match change {
            AmmlWindowChange::DPI => Self::Dpi,
            AmmlWindowChange::DISPLAY => Self::Display,
            AmmlWindowChange::MOVED => Self::Moved,
            _ => Self::Other,
        }
    }
}
