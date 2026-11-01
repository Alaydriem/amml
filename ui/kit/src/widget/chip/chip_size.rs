use crate::tokens::{SpacingToken, spacing};

/// The chip heights in Paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChipSize {
    /// Widgets, `Widget / Chip + Chip style`: `--spacing-control-small` high.
    #[default]
    Regular,
    /// Widgets, `Widget / Chip + Chip style`, row "HUD status": the status chips of a game HUD card.
    Hud,
}

impl ChipSize {
    /// Paper: Widgets, `Widget / Chip + Chip style`, row "HUD status": the chip height.
    const HUD_HEIGHT: f32 = 22.0;
    /// Paper: Widgets, `Widget / Chip + Chip style`, row "HUD status": the gap between the dot or icon and the text.
    const HUD_GAP: f32 = 6.0;

    pub fn height(self) -> f32 {
        match self {
            Self::Regular => <spacing::ControlSmall as SpacingToken>::VALUE,
            Self::Hud => Self::HUD_HEIGHT,
        }
    }

    /// The gap between the lead (an icon or a dot) and the text.
    pub fn gap(self) -> f32 {
        match self {
            Self::Regular => <spacing::Xs as SpacingToken>::VALUE,
            Self::Hud => Self::HUD_GAP,
        }
    }
}
