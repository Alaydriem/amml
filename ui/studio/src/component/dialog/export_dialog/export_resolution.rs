use amml_i18n::text::{Args, Text};

use crate::message::StudioUiMessage;

/// Paper: `Export dialog` (`971-0`), the Resolution row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportResolution {
    FullHd,
    Qhd,
    Uhd,
}

impl ExportResolution {
    pub const ALL: [Self; 3] = [Self::FullHd, Self::Qhd, Self::Uhd];

    pub fn label(self) -> Text {
        Text::message(match self {
            Self::FullHd => StudioUiMessage::Resolution1080,
            Self::Qhd => StudioUiMessage::Resolution1440,
            Self::Uhd => StudioUiMessage::Resolution4K,
        })
    }

    /// The width and height in pixels.
    pub fn pixels(self) -> (u64, u64) {
        match self {
            Self::FullHd => (1920, 1080),
            Self::Qhd => (2560, 1440),
            Self::Uhd => (3840, 2160),
        }
    }

    /// The size shown beside the row, as in "3840 × 2160".
    pub fn size(self) -> Text {
        let (width, height) = self.pixels();
        Text::format(StudioUiMessage::ResolutionSize, Args::new().uint("width", width).uint("height", height))
    }
}
