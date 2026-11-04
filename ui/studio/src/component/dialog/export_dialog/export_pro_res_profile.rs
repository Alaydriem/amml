use amml_i18n::text::Text;

use crate::message::StudioUiMessage;

/// Paper: `Export dialog` (`971-0`), the Profile row shown for ProRes 422: Standard or HQ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportProResProfile {
    Standard,
    Hq,
}

impl ExportProResProfile {
    pub const ALL: [Self; 2] = [Self::Standard, Self::Hq];

    pub fn label(self) -> Text {
        Text::message(match self {
            Self::Standard => StudioUiMessage::ProResStandard,
            Self::Hq => StudioUiMessage::ProResHq,
        })
    }
}
