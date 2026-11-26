use amml_runtime::ItemDrawKey;

/// What a failed contents open undoes: the copies and the nested lists it made.
#[derive(Default)]
pub(crate) struct OpenUndo {
    pub(crate) keys: Vec<ItemDrawKey>,
    pub(crate) lists: Vec<u64>,
}
