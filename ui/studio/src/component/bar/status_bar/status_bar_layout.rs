use amml_ui::tokens::{BreakpointToken, breakpoint};

/// Paper: Editor components, `Status bar` (`20D-0`). Full from `--breakpoint-small` up; Compact below it, where the
/// detail text goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusBarLayout {
    #[default]
    Full,
    Compact,
}

impl StatusBarLayout {
    pub fn for_width(width: f32) -> Self {
        if width < <breakpoint::Small as BreakpointToken>::VALUE { Self::Compact } else { Self::Full }
    }
}
