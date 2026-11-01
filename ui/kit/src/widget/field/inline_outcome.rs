/// What happened to an inline edit this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InlineOutcome {
    /// Not typing.
    Idle,
    /// Still typing.
    Editing,
    /// Enter or a click away: the typed text, to be checked by the field.
    Commit(String),
    /// Escape: the typed text is thrown away.
    Cancel,
}
