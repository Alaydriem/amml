/// The result of one `EventBus::dispatch`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DispatchReport {
    pub cancelled: bool,
    pub panicked_owners: Vec<usize>,
}
