/// Why a hold of the game clock ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeReleaseReason {
    /// The holder released it (the drop of its hold).
    Release,
    /// The world was left.
    WorldLeft,
    /// The holder was disabled.
    Disabled,
    /// No step came for the watchdog time of the hold.
    Watchdog,
}

impl TimeReleaseReason {
    /// The name the loader logs and the fake writes.
    pub fn name(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::WorldLeft => "world_left",
            Self::Disabled => "disabled",
            Self::Watchdog => "watchdog",
        }
    }
}
