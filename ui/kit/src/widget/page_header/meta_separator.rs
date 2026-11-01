use egui::{Painter, Pos2, Rect, vec2};

use crate::draw::{Corner, Draw};
use crate::tokens::{ColorToken, color};

/// Paper: Mod manager, `Widget / Page header`, the dot between meta items. A drawn part, never
/// a "·" in a string.
pub struct MetaSeparator;

impl MetaSeparator {
    /// Paper: Page header, the separator dot.
    pub const SIDE: f32 = 3.0;

    pub fn paint(painter: &Painter, centre: Pos2) {
        let rect = Rect::from_center_size(centre, vec2(Self::SIDE, Self::SIDE));
        Draw::fill(painter, rect, Corner::Full, <color::TextFaint as ColorToken>::VALUE);
    }
}
