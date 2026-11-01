use egui::{Color32, Painter, Rect};

use crate::tokens::{SpacingToken, spacing};

/// Paper: Assets, `Glyph / swatch dot`. The colour swatch in menus and the mode dot.
pub struct SwatchDot;

impl SwatchDot {
    /// Paints a filled dot of the swatch diameter, centred on `rect`.
    pub fn paint(painter: &Painter, rect: Rect, color: Color32) {
        painter.circle_filled(rect.center(), <spacing::Swatch as SpacingToken>::VALUE / 2.0, color);
    }
}
