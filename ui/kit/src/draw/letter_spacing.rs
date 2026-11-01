use crate::tokens::{TrackingToken, tracking};

/// The letter spacing of a text style. Section heads and small caps use `--tracking-section`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LetterSpacing {
    None,
    Section,
}

impl LetterSpacing {
    /// The extra space after each glyph, in points, at `size` points.
    pub fn points(self, size: f32) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Section => <tracking::Section as TrackingToken>::VALUE * size,
        }
    }
}
