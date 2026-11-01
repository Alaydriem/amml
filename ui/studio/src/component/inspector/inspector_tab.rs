use amml_i18n::text::Text;
use amml_ui::icon::Icon;

use crate::message::StudioUiMessage;

/// The inspector's tabs, in strip order. A selected track header with no selected key is a state of the Key tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectorTab {
    Key,
    Scene,
    Players,
}

impl InspectorTab {
    pub const ALL: [Self; 3] = [Self::Key, Self::Scene, Self::Players];

    pub fn label(self) -> Text {
        Text::message(match self {
            Self::Key => StudioUiMessage::InspectorKey,
            Self::Scene => StudioUiMessage::InspectorScene,
            Self::Players => StudioUiMessage::InspectorPlayers,
        })
    }

    /// The tab's icon on the inspector rail.
    pub fn icon(self) -> Icon {
        match self {
            Self::Key => Icon::Edit,
            Self::Scene => Icon::Video,
            Self::Players => Icon::People,
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|tab| *tab == self).unwrap_or(0)
    }
}
