/// One registered inline icon: its owner and its cell (the code point minus `U+E110`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct InlineIconEntry {
    pub(crate) owner: usize,
    pub(crate) cell: usize,
}
