use amml_ui::glyph::Glyph;
use egui::{Color32, Painter, Rect, pos2, vec2};

/// Paper: Assets, `Glyph / checkpoint tick` (`39A-0`). A small bar standing on the bottom of the rect, at its
/// centre: one recording checkpoint on the ruler. The caller gives the colour (`--color-checkpoint`).
pub struct CheckpointTick;

impl CheckpointTick {
    /// Paper: the tick is 3 × 6 with a 1 px corner.
    pub const WIDTH: f32 = 3.0;
    pub const HEIGHT: f32 = 6.0;
    const CORNER: u8 = 1;
}

impl Glyph for CheckpointTick {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        let tick = Rect::from_center_size(
            pos2(rect.center().x, rect.bottom() - Self::HEIGHT / 2.0),
            vec2(Self::WIDTH, Self::HEIGHT),
        );
        painter.rect_filled(tick, Self::CORNER, color);
    }
}
