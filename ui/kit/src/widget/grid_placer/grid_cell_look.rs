use amml_i18n::text::Text;

/// Paper: Widgets, `Widget / Grid placer`, the columns of the Cell row. The text is the piece name.
#[derive(Debug, Clone)]
pub enum GridCellLook {
    /// Dashed `--color-line-strong`, the cell label only.
    Empty,
    /// A piece is dragged over the empty cell: "Drop here" on `--color-accent-soft`, dashed accent.
    DropTarget,
    /// The keyboard reached the empty cell while a piece is held: "Place here", as the drop target.
    PlaceHere,
    Filled(Text),
    /// The pointer is on the filled cell: `--color-bg-hover` and the unplace button.
    FilledHover(Text),
    /// A piece is dragged over the filled cell: "Taken" on `--color-danger-soft`.
    Blocked(Text),
}
