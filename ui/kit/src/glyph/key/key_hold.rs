use egui::epaint::RectShape;
use egui::{Color32, CornerRadius, Painter, Rect, StrokeKind, Vec2};

use super::{KeyLook, KeyMark};
use crate::glyph::Glyph;

/// Paper: Assets, `Glyph / key / hold (square)`. A key that holds its value to the next key.
pub struct KeyHold {
    pub look: KeyLook,
}

impl KeyHold {
    /// Paper: the side of the square at rest.
    pub const REST_SIDE: f32 = 9.6;
    /// Paper: the side of the square when selected.
    pub const SELECTED_SIDE: f32 = 12.0;
    // Paper draws a 1.5 px corner; egui keeps corner radii in whole pixels
    const CORNER: u8 = 2;
}

impl Glyph for KeyHold {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        let centre = rect.center();
        KeyMark::halo(painter, centre, color, self.look);
        let side = match self.look {
            KeyLook::Rest => Self::REST_SIDE,
            KeyLook::Selected => Self::SELECTED_SIDE,
        };
        let square = Rect::from_center_size(centre, Vec2::splat(side));
        let corner = CornerRadius::same(Self::CORNER);
        painter.add(RectShape::new(square, corner, color, KeyMark::outline(self.look), StrokeKind::Middle));
    }
}
