use egui::Color32;

use crate::icon::Icon;
use crate::tokens::rule::SoftFill;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Mod manager, `Widget / Banner`, the four tones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BannerTone {
    Info,
    Warning,
    Danger,
    Success,
}

impl BannerTone {
    /// Paper: Banner, the top and bottom padding of the coloured tones.
    const TONE_PADDING: f32 = 14.0;

    pub fn fill(self) -> Color32 {
        match self {
            Self::Info => <color::BgApp as ColorToken>::VALUE,
            Self::Warning => <color::WarningSoft as ColorToken>::VALUE,
            Self::Danger => <color::DangerSoft as ColorToken>::VALUE,
            Self::Success => SoftFill::of(<color::Success as ColorToken>::VALUE),
        }
    }

    pub fn border(self) -> Color32 {
        match self {
            Self::Info => <color::Line as ColorToken>::VALUE,
            Self::Warning | Self::Danger | Self::Success => self.color(),
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Self::Info => Icon::Info,
            Self::Warning => Icon::Warning,
            Self::Danger => Icon::Blocked,
            Self::Success => Icon::Check,
        }
    }

    pub fn icon_color(self) -> Color32 {
        match self {
            Self::Info => <color::TextDim as ColorToken>::VALUE,
            Self::Warning | Self::Danger | Self::Success => self.color(),
        }
    }

    pub fn title_color(self) -> Color32 {
        match self {
            Self::Info => <color::Text as ColorToken>::VALUE,
            Self::Warning | Self::Danger | Self::Success => self.color(),
        }
    }

    /// The top and bottom padding.
    pub fn padding(self) -> f32 {
        match self {
            Self::Info => <spacing::L as SpacingToken>::VALUE,
            Self::Warning | Self::Danger | Self::Success => Self::TONE_PADDING,
        }
    }

    /// The gap between the icon, the text and the action.
    pub fn gap(self) -> f32 {
        match self {
            Self::Info => <spacing::L as SpacingToken>::VALUE,
            Self::Warning | Self::Danger | Self::Success => <spacing::M as SpacingToken>::VALUE,
        }
    }

    // the tone's role colour
    fn color(self) -> Color32 {
        match self {
            Self::Info => <color::TextDim as ColorToken>::VALUE,
            Self::Warning => <color::Warning as ColorToken>::VALUE,
            Self::Danger => <color::Danger as ColorToken>::VALUE,
            Self::Success => <color::Success as ColorToken>::VALUE,
        }
    }
}
