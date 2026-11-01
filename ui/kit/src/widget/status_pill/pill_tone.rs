use egui::Color32;

use crate::tokens::rule::SoftFill;
use crate::tokens::{ColorToken, color};

/// Paper: Widgets, `Widget / Status pill`. A pill's colour; text and dot share it, and the fill is
/// its soft form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PillTone {
    /// Queued.
    Dim,
    /// Cancelled, draft, read-only.
    Faint,
    /// Ready, busy: `--color-accent-soft` fill, `--color-accent-hover` text, `--color-accent` dot.
    Accent,
    Success,
    Warning,
    Danger,
    /// A token colour the caller picks from a selector enum (scene, mode or coverage colours in
    /// amml-studio-ui), with `SoftFill::of` behind it.
    Role(Color32),
}

impl PillTone {
    pub fn fill(self) -> Color32 {
        match self {
            Self::Dim => SoftFill::of(<color::TextDim as ColorToken>::VALUE),
            Self::Faint => SoftFill::of(<color::TextFaint as ColorToken>::VALUE),
            Self::Accent => <color::AccentSoft as ColorToken>::VALUE,
            Self::Success => SoftFill::of(<color::Success as ColorToken>::VALUE),
            Self::Warning => <color::WarningSoft as ColorToken>::VALUE,
            Self::Danger => <color::DangerSoft as ColorToken>::VALUE,
            Self::Role(color) => SoftFill::of(color),
        }
    }

    pub fn text(self) -> Color32 {
        match self {
            Self::Dim => <color::TextDim as ColorToken>::VALUE,
            Self::Faint => <color::TextFaint as ColorToken>::VALUE,
            Self::Accent => <color::AccentHover as ColorToken>::VALUE,
            Self::Success => <color::Success as ColorToken>::VALUE,
            Self::Warning => <color::Warning as ColorToken>::VALUE,
            Self::Danger => <color::Danger as ColorToken>::VALUE,
            Self::Role(color) => color,
        }
    }

    pub fn dot(self) -> Color32 {
        match self {
            Self::Accent => <color::Accent as ColorToken>::VALUE,
            Self::Dim | Self::Faint | Self::Success | Self::Warning | Self::Danger | Self::Role(_) => self.text(),
        }
    }
}
