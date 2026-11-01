use egui::FontId;

use super::{LetterSpacing, TypeSize};
use crate::font::FontFace;

/// How a run of text looks: a type-scale size, a face, and its letter spacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextStyle {
    pub size: TypeSize,
    pub face: FontFace,
    pub spacing: LetterSpacing,
}

impl TextStyle {
    pub const fn new(size: TypeSize, face: FontFace) -> Self {
        Self { size, face, spacing: LetterSpacing::None }
    }

    /// The same style with `--tracking-section` letter spacing.
    pub const fn tracked(self) -> Self {
        Self { spacing: LetterSpacing::Section, ..self }
    }

    pub fn font_id(self) -> FontId {
        self.face.id(self.size.points())
    }
}
