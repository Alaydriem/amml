use amml_abi::AmmlUiTable;

/// One `ChatMessageAdded` call in progress: the address of the event the listener got, the loader ui table and the module
/// of the listener.
#[derive(Clone, Copy)]
pub(crate) struct LentChatEvent {
    pub(crate) event: usize,
    pub(crate) table: AmmlUiTable,
    pub(crate) module: &'static str,
}
