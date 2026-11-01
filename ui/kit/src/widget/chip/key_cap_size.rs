use crate::draw::{TextStyle, TypeSize};
use crate::font::FontFace;
use crate::tokens::{SpacingToken, spacing};

/// The key cap looks in Paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum KeyCapSize {
    /// Chip sheet, "Key chord": `--spacing-status-bar` high, help text, border and an under-edge.
    #[default]
    Regular,
    /// Tooltip sheet, "With shortcut": `--spacing-control-small` high, micro semibold, no border.
    Small,
    /// Chip sheet, row "HUD key": a fixed 32 by 28 cap with a border, label size text or an arrow icon.
    Hud,
    /// Chip sheet, row "HUD hint": 18 high with a border, micro text.
    HudHint,
}

impl KeyCapSize {
    /// Paper: Widgets, `Widget / Chip + Chip style`, row "HUD key": the cap width.
    const HUD_WIDTH: f32 = 32.0;
    /// Paper: Widgets, `Widget / Chip + Chip style`, row "HUD key": the cap height.
    const HUD_HEIGHT: f32 = 28.0;
    /// Paper: Widgets, `Widget / Chip + Chip style`, row "HUD hint": the cap height, the line height of its text.
    const HINT_HEIGHT: f32 = 18.0;
    /// Paper: Widgets, `Widget / Chip + Chip style`, row "HUD hint": the padding at the left and right.
    const HINT_PAD: f32 = 5.0;

    pub fn height(self) -> f32 {
        match self {
            Self::Regular => <spacing::StatusBar as SpacingToken>::VALUE,
            Self::Small => <spacing::ControlSmall as SpacingToken>::VALUE,
            Self::Hud => Self::HUD_HEIGHT,
            Self::HudHint => Self::HINT_HEIGHT,
        }
    }

    /// The narrowest a cap gets (a one-letter key).
    pub fn min_width(self) -> f32 {
        match self {
            Self::Regular => <spacing::Row as SpacingToken>::VALUE,
            Self::Small | Self::HudHint => 0.0,
            Self::Hud => Self::HUD_WIDTH,
        }
    }

    /// The padding at the left and right of the label.
    pub fn padding(self) -> f32 {
        match self {
            Self::Regular | Self::Small | Self::Hud => <spacing::S as SpacingToken>::VALUE,
            Self::HudHint => Self::HINT_PAD,
        }
    }

    pub fn text(self) -> TextStyle {
        match self {
            Self::Regular => TextStyle::new(TypeSize::Help, FontFace::Regular),
            Self::Small => TextStyle::new(TypeSize::Micro, FontFace::Semibold),
            Self::Hud => TextStyle::new(TypeSize::Label, FontFace::Regular),
            Self::HudHint => TextStyle::new(TypeSize::Micro, FontFace::Regular),
        }
    }
}
