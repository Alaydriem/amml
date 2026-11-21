use amml_abi::AmmlRenderTable;

/// One `WorldRendered` call in progress: the address of the event the listener got, the loader render table and the
/// module of the listener.
#[derive(Clone, Copy)]
pub(crate) struct LentRenderEvent {
    pub(crate) event: usize,
    pub(crate) table: AmmlRenderTable,
    pub(crate) module: &'static str,
}
