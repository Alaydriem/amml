use crate::tokens::{SpacingToken, spacing};

/// Paper: Widgets, `Glyph / Spinner`. The 14 px spinner sits on text and in buttons; the 20 px one stands alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpinnerSize {
    Small,
    Regular,
}

impl SpinnerSize {
    pub fn side(self) -> f32 {
        match self {
            Self::Small => <spacing::SpinnerSmall as SpacingToken>::VALUE,
            Self::Regular => <spacing::Spinner as SpacingToken>::VALUE,
        }
    }

    /// The radius of the track and arc centre line.
    pub fn radius(self) -> f32 {
        match self {
            Self::Small => 5.5,
            Self::Regular => 8.0,
        }
    }

    pub fn stroke(self) -> f32 {
        match self {
            Self::Small => 1.75,
            Self::Regular => 2.0,
        }
    }
}
