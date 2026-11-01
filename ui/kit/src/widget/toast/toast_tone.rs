use egui::Color32;

use crate::icon::Icon;
use crate::tokens::rule::SoftFill;
use crate::tokens::{ColorToken, color};

/// Paper: Mod manager, `Widget / Toast`, the four tones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToastTone {
    Info,
    Success,
    Warning,
    Danger,
}

impl ToastTone {
    pub fn icon(self) -> Icon {
        match self {
            Self::Info => Icon::Info,
            Self::Success => Icon::Check,
            Self::Warning => Icon::Warning,
            Self::Danger => Icon::Blocked,
        }
    }

    /// The icon and the countdown fill.
    pub fn color(self) -> Color32 {
        match self {
            Self::Info => <color::Accent as ColorToken>::VALUE,
            Self::Success => <color::Success as ColorToken>::VALUE,
            Self::Warning => <color::Warning as ColorToken>::VALUE,
            Self::Danger => <color::Danger as ColorToken>::VALUE,
        }
    }

    /// The round tile behind the icon.
    pub fn soft(self) -> Color32 {
        match self {
            Self::Info => <color::AccentSoft as ColorToken>::VALUE,
            Self::Success => SoftFill::of(<color::Success as ColorToken>::VALUE),
            Self::Warning => <color::WarningSoft as ColorToken>::VALUE,
            Self::Danger => <color::DangerSoft as ColorToken>::VALUE,
        }
    }
}
