use amml_i18n::text::Text;

/// Paper: Mod manager, `Widget / Table header + Table row`, the row kinds.
#[derive(Debug, Clone)]
pub enum TableRowKind {
    Plain,
    /// Waiting for a key: an accent box, and the chord cell shows "Press a key" and "Esc cancels".
    Capturing,
    /// The chord is used twice: a danger box, and this reason under the action ("Also bound by
    /// Light Level").
    Conflict(Text),
    /// The row cannot change: the whole row at `--opacity-muted`, and this reason under the action.
    Disabled(Text),
}

impl TableRowKind {
    /// Capturing and conflict rows are drawn as a box, `--spacing-xs` from the rows around them.
    pub fn boxed(&self) -> bool {
        matches!(self, Self::Capturing | Self::Conflict(_))
    }
}
