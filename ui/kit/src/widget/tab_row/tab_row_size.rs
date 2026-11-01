use crate::draw::{TextStyle, TypeSize};
use crate::font::FontFace;
use crate::tokens::{SpacingToken, spacing};

/// Paper: Widgets, `Widget / Tab row`, rows "Inspector" and "Compact".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TabRowSize {
    /// `--spacing-button` high, `--spacing-m` padding, help text: the inspector header.
    #[default]
    Inspector,
    /// `--spacing-row` high, `--spacing-s` padding, note text: dialog and library headers.
    Compact,
}

impl TabRowSize {
    pub fn height(self) -> f32 {
        match self {
            Self::Inspector => <spacing::Button as SpacingToken>::VALUE,
            Self::Compact => <spacing::Row as SpacingToken>::VALUE,
        }
    }

    pub fn padding(self) -> f32 {
        match self {
            Self::Inspector => <spacing::M as SpacingToken>::VALUE,
            Self::Compact => <spacing::S as SpacingToken>::VALUE,
        }
    }

    pub fn text(self) -> TextStyle {
        match self {
            Self::Inspector => TextStyle::new(TypeSize::Help, FontFace::Semibold),
            Self::Compact => TextStyle::new(TypeSize::Note, FontFace::Semibold),
        }
    }
}
