use amml_abi::AmmlNetworkTable;

/// One network event call in progress: the address of the event the listener got and the loader network table.
#[derive(Clone, Copy)]
pub(crate) struct LentNetworkEvent {
    pub(crate) event: usize,
    pub(crate) table: AmmlNetworkTable,
}
