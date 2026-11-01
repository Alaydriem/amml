use crate::draw::{TextStyle, TypeSize};
use crate::font::FontFace;
use crate::tokens::{SpacingToken, spacing};

/// Paper: Widgets, `Widget / Value row`, the two heights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueRowSize {
    /// `--spacing-row` high, help-size label: the Library facts.
    Large,
    /// `--spacing-control-small` high, note-size label: the Scene tab facts.
    Compact,
}

impl ValueRowSize {
    pub fn height(self) -> f32 {
        match self {
            Self::Large => <spacing::Row as SpacingToken>::VALUE,
            Self::Compact => <spacing::ControlSmall as SpacingToken>::VALUE,
        }
    }

    pub fn label(self) -> TextStyle {
        match self {
            Self::Large => TextStyle::new(TypeSize::Help, FontFace::Regular),
            Self::Compact => TextStyle::new(TypeSize::Note, FontFace::Regular),
        }
    }
}
