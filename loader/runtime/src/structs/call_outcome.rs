/// What one listener call reports to `EventBus::dispatch`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallOutcome {
    Done,
    Cancelled,
    Panicked,
}
