use egui::Color32;

use crate::tokens::rule::SoftFill;
use crate::tokens::{ColorToken, color};

/// Paper: Widgets, `Widget / Chip + Chip style`, row "Style". A display chip's look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChipTone {
    Neutral,
    Accent,
    Warning,
    Danger,
    Success,
    /// No fill and a 1 px `--color-line-strong` border: save status, "applies after reload",
    /// capability chips.
    Outline,
}

impl ChipTone {
    pub fn fill(self) -> Color32 {
        match self {
            Self::Neutral => <color::BgField as ColorToken>::VALUE,
            Self::Accent => <color::AccentSoft as ColorToken>::VALUE,
            Self::Warning => <color::WarningSoft as ColorToken>::VALUE,
            Self::Danger => <color::DangerSoft as ColorToken>::VALUE,
            Self::Success => SoftFill::of(<color::Success as ColorToken>::VALUE),
            Self::Outline => Color32::TRANSPARENT,
        }
    }

    pub fn text(self) -> Color32 {
        match self {
            Self::Neutral | Self::Outline => <color::TextDim as ColorToken>::VALUE,
            Self::Accent => <color::AccentHover as ColorToken>::VALUE,
            Self::Warning => <color::Warning as ColorToken>::VALUE,
            Self::Danger => <color::Danger as ColorToken>::VALUE,
            Self::Success => <color::Success as ColorToken>::VALUE,
        }
    }

    pub fn border(self) -> Option<Color32> {
        match self {
            Self::Outline => Some(<color::LineStrong as ColorToken>::VALUE),
            Self::Neutral | Self::Accent | Self::Warning | Self::Danger | Self::Success => None,
        }
    }
}
