/// What the rename dialog asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameAction {
    /// Apply the new name (never empty).
    Confirm(String),
    /// The close button, Escape or a click outside.
    Cancel,
}
