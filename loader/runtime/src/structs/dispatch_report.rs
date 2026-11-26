/// The result of one `EventBus::dispatch`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DispatchReport {
    pub cancelled: bool,
    /// The owner of the first listener that cancelled.
    pub cancelled_by: Option<usize>,
    pub panicked_owners: Vec<usize>,
}
