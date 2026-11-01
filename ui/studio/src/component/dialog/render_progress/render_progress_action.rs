/// What the render progress card asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderProgressAction {
    /// Stop the running render: the Cancel button, the close mark or Escape.
    Cancel,
    /// Put away the card of a render that is over: the Close button, the close mark or Escape.
    Close,
    /// Open the render queue dialog.
    OpenQueue,
}
