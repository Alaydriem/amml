/// Paper: Widgets, `Widget / Grid placer`, the columns of the Piece chip row, and the lifted ghost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridChipLook {
    Rest,
    Hover,
    /// The chip under the pointer while it is dragged: `--color-bg-active`, accent border, shadow.
    Dragging,
    Focused,
    /// Picked up with Enter for the keyboard route: `--color-accent-soft`, accent border, focus ring.
    Held,
    /// The chip's place in the tray while it is dragged: dashed, the content at `--opacity-muted`.
    Ghost,
}
