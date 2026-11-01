use amml_i18n::text::{Args, Text};

use crate::message::StudioUiMessage;

/// Paper: `Export dialog` (`971-0`), the Frame rate row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFrameRate {
    TwentyFour,
    Thirty,
    Sixty,
}

impl ExportFrameRate {
    pub const ALL: [Self; 3] = [Self::TwentyFour, Self::Thirty, Self::Sixty];

    pub fn frames_per_second(self) -> u64 {
        match self {
            Self::TwentyFour => 24,
            Self::Thirty => 30,
            Self::Sixty => 60,
        }
    }

    pub fn label(self) -> Text {
        Text::format(StudioUiMessage::Count, Args::new().uint("count", self.frames_per_second()))
    }
}
