use std::f32::consts::{FRAC_PI_2, TAU};

use egui::{Color32, Painter, Rect, Shape, Stroke, vec2};

use super::SpinnerSize;
use crate::tokens::{OpacityToken, opacity};

/// Paper: Widgets, `Glyph / Spinner`. A faint track circle with a quarter arc on it. The caller animates `turn`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spinner {
    pub size: SpinnerSize,
    /// How far the arc has turned, as a fraction of a full turn. Any value works; it wraps.
    pub turn: f32,
}

impl Spinner {
    const ARC_SEGMENTS: usize = 12;

    /// Paints the track, then the arc, centred on `rect`.
    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        let centre = rect.center();
        let radius = self.size.radius();
        let width = self.size.stroke();
        let track = color.gamma_multiply(<opacity::Track as OpacityToken>::VALUE);
        painter.circle_stroke(centre, radius, Stroke::new(width, track));

        let start = self.turn.rem_euclid(1.0) * TAU - FRAC_PI_2;
        let points = (0..=Self::ARC_SEGMENTS)
            .map(|step| {
                let angle = start + FRAC_PI_2 * step as f32 / Self::ARC_SEGMENTS as f32;
                centre + vec2(angle.cos(), angle.sin()) * radius
            })
            .collect();
        painter.add(Shape::line(points, Stroke::new(width, color)));
    }
}
