use crate::MeshKind;

/// One live mesh of the book.
pub(crate) struct MeshEntry {
    pub(crate) owner: usize,
    /// The address of the loader's game mesh object.
    pub(crate) handle: usize,
    pub(crate) kind: MeshKind,
}
