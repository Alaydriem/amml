use egui::Color32;

use crate::state::StateStyle;
use crate::tokens::{ColorToken, color};

/// Paper: Widgets, `Widget / Text button`, one row per kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Outline,
    Danger,
}

impl ButtonKind {
    pub fn style(self) -> StateStyle {
        match self {
            Self::Primary => StateStyle::Accent,
            Self::Secondary => StateStyle::Field,
            Self::Ghost | Self::Outline => StateStyle::Clear,
            Self::Danger => StateStyle::Danger,
        }
    }

    /// The label and icon colour, the same in every state.
    pub fn text(self) -> Color32 {
        match self {
            Self::Primary | Self::Danger => <color::OnAccent as ColorToken>::VALUE,
            Self::Secondary | Self::Ghost | Self::Outline => <color::Text as ColorToken>::VALUE,
        }
    }

    /// The 1 px border inside the button, the same in every state.
    pub fn border(self) -> Option<Color32> {
        match self {
            Self::Primary | Self::Danger => Some(<color::LineOverlay as ColorToken>::VALUE),
            Self::Outline => Some(<color::LineStrong as ColorToken>::VALUE),
            Self::Secondary | Self::Ghost => None,
        }
    }
}
