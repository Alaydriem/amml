use egui::{CornerRadius, Rect};

use crate::tokens::{RadiusToken, radius};

/// The corner of a painted part, picked from the radius tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Corner {
    None,
    Small,
    Control,
    Panel,
    /// `--radius-full`: half the shorter side, so the part is fully round.
    Full,
}

impl Corner {
    pub fn radius(self, rect: Rect) -> CornerRadius {
        let points = match self {
            Self::None => 0.0,
            Self::Small => <radius::Small as RadiusToken>::VALUE,
            Self::Control => <radius::Control as RadiusToken>::VALUE,
            Self::Panel => <radius::Panel as RadiusToken>::VALUE,
            Self::Full => rect.width().min(rect.height()) / 2.0,
        };
        // Clamped to 0..=255 first, so the cast cannot truncate.
        CornerRadius::same(points.round().clamp(0.0, 255.0) as u8)
    }
}
