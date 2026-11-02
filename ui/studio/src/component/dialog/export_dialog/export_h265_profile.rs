use amml_i18n::text::Text;

use crate::message::StudioUiMessage;

/// Paper: `Dialog / Export dialog / H.265 profile (mock)`, the Profile row shown for H.265: Main (8 bit) or Main 10
/// (10 bit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportH265Profile {
    Main,
    Main10,
}

impl ExportH265Profile {
    pub const ALL: [Self; 2] = [Self::Main, Self::Main10];

    pub fn label(self) -> Text {
        Text::message(match self {
            Self::Main => StudioUiMessage::H265Main,
            Self::Main10 => StudioUiMessage::H265Main10,
        })
    }
}
