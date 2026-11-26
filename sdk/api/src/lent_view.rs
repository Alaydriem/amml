use amml_abi::{AmmlInventoryTable, AmmlUiTable};

/// One `ScreenRender` call in progress: the address of the event the listener got, the loader ui table, the loader
/// inventory table (the stacks of a container screen are read through it) and the module of the listener.
#[derive(Clone, Copy)]
pub(crate) struct LentView {
    pub(crate) event: usize,
    pub(crate) table: AmmlUiTable,
    pub(crate) inventory: Option<AmmlInventoryTable>,
    pub(crate) module: &'static str,
}
