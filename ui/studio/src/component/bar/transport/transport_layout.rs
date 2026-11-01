use amml_ui::tokens::{BreakpointToken, breakpoint};

/// Paper: Editor components, `Transport` (`UW-0`). Full from `--breakpoint-small` up; Compact below it, where Add camera
/// key and Fit become icons and the zoom slider is wider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportLayout {
    #[default]
    Full,
    Compact,
}

impl TransportLayout {
    pub fn for_width(width: f32) -> Self {
        if width < <breakpoint::Small as BreakpointToken>::VALUE { Self::Compact } else { Self::Full }
    }
}
