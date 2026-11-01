use crate::tokens::{RadiusToken, TextSizeToken, radius, text_size};

/// Paper: Assets, `Brand / DV mark`. Regular sits in the top bar and the library header; Small sits on the capture card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DvMarkSize {
    Regular,
    Small,
}

impl DvMarkSize {
    pub fn side(self) -> f32 {
        match self {
            Self::Regular => 26.0,
            Self::Small => 18.0,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Self::Regular => <radius::Control as RadiusToken>::VALUE,
            Self::Small => <radius::Small as RadiusToken>::VALUE,
        }
    }

    pub fn text_points(self) -> f32 {
        match self {
            Self::Regular => <text_size::Micro as TextSizeToken>::VALUE,
            Self::Small => 8.0,
        }
    }

    /// Extra space between the two letters, in points.
    pub fn tracking(self) -> f32 {
        match self {
            Self::Regular => 0.4,
            Self::Small => 0.0,
        }
    }
}
