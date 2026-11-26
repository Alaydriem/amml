use crate::GhostEntity;

/// One live ghost of the book.
pub(crate) struct GhostEntityEntry {
    pub(crate) owner: usize,
    pub(crate) ghost: GhostEntity,
}
