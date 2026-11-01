use amml_i18n::text::Text;

use crate::message::StudioUiMessage;

/// Paper: `Export dialog` (`971-0`), the Format row: the two codecs the render encodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    H264,
    H265,
}

impl ExportFormat {
    pub const ALL: [Self; 2] = [Self::H264, Self::H265];

    pub fn label(self) -> Text {
        Text::message(match self {
            Self::H264 => StudioUiMessage::FormatH264,
            Self::H265 => StudioUiMessage::FormatH265,
        })
    }
}
