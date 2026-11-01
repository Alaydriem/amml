use amml_i18n::text::Text;

/// Paper: Widgets, `Widget / Loading state`.
#[derive(Debug, Clone)]
pub enum LoadingKind {
    /// Placeholder rows for a list that is loading, `--spacing-lane` high each.
    Skeleton { rows: usize },
    /// One status line: a small spinner and the label.
    Inline(Text),
}
