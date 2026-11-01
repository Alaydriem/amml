use egui::{Painter, Response};

use crate::tokens::{OpacityToken, opacity};

/// The one state every interactive widget reads its look from (Paper: the REST, HOVER, PRESSED,
/// FOCUS and DISABLED columns of every widget sheet).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionState {
    Rest,
    Hover,
    Pressed,
    Focused,
    Disabled,
}

impl InteractionState {
    /// The state of `response`. Precedence: Disabled, Pressed, Hover, Focused, Rest. The focus ring
    /// shows only in Focused, so a focused control under the pointer shows its hover look.
    pub fn of(response: &Response, enabled: bool) -> Self {
        if !enabled {
            Self::Disabled
        } else if response.is_pointer_button_down_on() || response.dragged() {
            Self::Pressed
        } else if response.hovered() {
            Self::Hover
        } else if response.has_focus() {
            Self::Focused
        } else {
            Self::Rest
        }
    }

    /// The state for a part drawn inside a control that already took the focus ring and the
    /// disabled opacity: Focused and Disabled become Rest, so neither is applied twice.
    pub fn inner(self) -> Self {
        match self {
            Self::Focused | Self::Disabled => Self::Rest,
            Self::Rest | Self::Hover | Self::Pressed => self,
        }
    }

    /// A painter for this state: the whole control at `--opacity-muted` when Disabled.
    pub fn painter(self, painter: &Painter) -> Painter {
        let mut painter = painter.clone();
        if self == Self::Disabled {
            painter.multiply_opacity(<opacity::Muted as OpacityToken>::VALUE);
        }
        painter
    }
}
