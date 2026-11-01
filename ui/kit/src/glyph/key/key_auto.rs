use egui::{Color32, Painter, Rect};

use super::{KeyLook, KeyMark};
use crate::glyph::Glyph;

/// Paper: Assets, `Glyph / key / auto (circle)`. A key whose curve is chosen for it.
pub struct KeyAuto {
    pub look: KeyLook,
}

impl KeyAuto {
    /// Paper: the circle radius at rest.
    pub const REST_RADIUS: f32 = 5.7;
    /// Paper: the circle radius when selected.
    pub const SELECTED_RADIUS: f32 = 7.1;
}

impl Glyph for KeyAuto {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        let centre = rect.center();
        KeyMark::halo(painter, centre, color, self.look);
        let radius = match self.look {
            KeyLook::Rest => Self::REST_RADIUS,
            KeyLook::Selected => Self::SELECTED_RADIUS,
        };
        painter.circle(centre, radius, color, KeyMark::outline(self.look));
    }
}
