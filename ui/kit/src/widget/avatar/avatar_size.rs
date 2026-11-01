use crate::draw::{Corner, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::tokens::{SpacingToken, spacing};

/// Paper: Mod manager, `Widget / Avatar`, the three sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AvatarSize {
    /// `--spacing-tile-large` square, `--radius-panel`: the page header.
    Large,
    /// 40 square, `--radius-control`: a list row.
    #[default]
    Regular,
    /// 32 square, `--radius-control`: the players tab.
    Small,
}

impl AvatarSize {
    /// Paper: Avatar sheet, the Regular side.
    const REGULAR: f32 = 40.0;
    /// Paper: Avatar sheet, the Small side.
    const SMALL: f32 = 32.0;

    pub fn side(self) -> f32 {
        match self {
            Self::Large => <spacing::TileLarge as SpacingToken>::VALUE,
            Self::Regular => Self::REGULAR,
            Self::Small => Self::SMALL,
        }
    }

    pub fn corner(self) -> Corner {
        match self {
            Self::Large => Corner::Panel,
            Self::Regular | Self::Small => Corner::Control,
        }
    }

    pub fn letter(self) -> TextStyle {
        match self {
            Self::Large => TextStyle::new(TypeSize::Title, FontFace::Semibold),
            Self::Regular | Self::Small => TextStyle::new(TypeSize::Label, FontFace::Semibold),
        }
    }
}
