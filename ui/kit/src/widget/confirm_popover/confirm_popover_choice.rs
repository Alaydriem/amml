/// What the confirm popover was answered with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfirmPopoverChoice {
    /// The danger button, or Enter.
    Confirm,
    /// The other button, or Escape.
    Cancel,
}
