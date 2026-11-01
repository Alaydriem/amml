/// What a click in the undo history asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoHistoryAction {
    /// Undo or redo to just after the step at this index of `UndoHistoryView::steps`.
    JumpTo(usize),
}
