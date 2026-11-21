use amml_abi::AmmlRequestOutcome;

/// The end of one request: its id, how it ended, the moves the server accepted and the moves it had.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestDone {
    pub id: u64,
    pub outcome: AmmlRequestOutcome,
    pub accepted: u16,
    pub total: u16,
}
