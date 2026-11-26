/// One live view of the book.
#[derive(Clone, Copy)]
pub(crate) struct ViewEntry {
    pub(crate) id: u64,
    pub(crate) controller: usize,
}
