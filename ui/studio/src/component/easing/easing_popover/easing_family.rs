use amml_i18n::text::Text;

use crate::message::StudioUiMessage;

/// Paper: `Easing popover` (`7J6-0`), the segmented control over the preset grid. Which presets the grid shows. A
/// preset's family is data, not a prefix of its name (spec 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EasingFamily {
    All,
    In,
    Out,
    InOut,
    Other,
}

impl EasingFamily {
    pub const ALL: [Self; 5] = [Self::All, Self::In, Self::Out, Self::InOut, Self::Other];

    pub fn label(self) -> Text {
        Text::message(match self {
            Self::All => StudioUiMessage::FamilyAll,
            Self::In => StudioUiMessage::FamilyIn,
            Self::Out => StudioUiMessage::FamilyOut,
            Self::InOut => StudioUiMessage::FamilyInOut,
            Self::Other => StudioUiMessage::FamilyOther,
        })
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|family| *family == self).unwrap_or(0)
    }

    /// True when a preset of `family` shows under this filter.
    pub fn shows(self, family: EasingFamily) -> bool {
        self == Self::All || self == family
    }
}
