/// What the confirm dialog asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Confirm,
    /// Cancel, the close button, Escape or a click outside.
    Cancel,
}
