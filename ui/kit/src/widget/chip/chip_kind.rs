use super::ChipTone;

/// What a chip is for: a filter the user clicks, or a label that only shows a tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChipKind {
    /// Clickable; selected filters lead with a check.
    Filter { selected: bool },
    /// Not interactive.
    Display(ChipTone),
}
