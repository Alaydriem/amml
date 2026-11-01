use crate::draw::Corner;
use crate::icon::IconSize;
use crate::tokens::{SpacingToken, spacing};

/// Paper: Widgets, `Widget / Icon button`, row "Size".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IconButtonSize {
    /// `--spacing-row` square, 16 px icon, `--radius-control`.
    #[default]
    Regular,
    /// `--spacing-control-small` square, 12 px icon, `--radius-small`.
    Small,
}

impl IconButtonSize {
    pub fn side(self) -> f32 {
        match self {
            Self::Regular => <spacing::Row as SpacingToken>::VALUE,
            Self::Small => <spacing::ControlSmall as SpacingToken>::VALUE,
        }
    }

    pub fn icon(self) -> IconSize {
        match self {
            Self::Regular => IconSize::Regular,
            Self::Small => IconSize::Small,
        }
    }

    pub fn corner(self) -> Corner {
        match self {
            Self::Regular => Corner::Control,
            Self::Small => Corner::Small,
        }
    }
}
