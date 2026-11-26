/// Why the time hold book refuses a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TimeHoldRefusal {
    #[error("a mod holds the game clock")]
    Taken,
    #[error("the hold ended")]
    Stale,
}
