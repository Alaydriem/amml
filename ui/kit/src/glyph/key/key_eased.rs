use egui::{Color32, Painter, Rect};

use super::{KeyLook, KeyMark};
use crate::glyph::Glyph;

/// Paper: Assets, `Glyph / key / eased (diamond)`. A key with a named easing curve to the next key.
pub struct KeyEased {
    pub look: KeyLook,
}

impl KeyEased {
    /// Paper: the distance from the centre to a corner at rest.
    pub const REST_HALF: f32 = 7.0;
    /// Paper: the distance from the centre to a corner when selected.
    pub const SELECTED_HALF: f32 = 8.5;
}

impl Glyph for KeyEased {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        let centre = rect.center();
        KeyMark::halo(painter, centre, color, self.look);
        let half = match self.look {
            KeyLook::Rest => Self::REST_HALF,
            KeyLook::Selected => Self::SELECTED_HALF,
        };
        KeyMark::diamond(painter, centre, half, color, KeyMark::outline(self.look));
    }
}
