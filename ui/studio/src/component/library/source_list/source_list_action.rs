/// A click in the source list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceListAction {
    /// The add folder button.
    Add,
    /// The remove button of the source at this place in the view's list.
    Remove(usize),
}
