use egui::{Color32, Painter, Rect};

use super::{KeyEased, KeyLook};
use crate::glyph::Glyph;
use crate::tokens::{ColorToken, color};

/// Paper: Assets, `Glyph / key / custom (diamond + dot)`. A key with a custom curve: the eased diamond with a dot.
pub struct KeyCustom {
    pub look: KeyLook,
}

impl KeyCustom {
    /// Paper: the radius of the `--color-bg-app` dot at the centre.
    pub const DOT_RADIUS: f32 = 2.0;
}

impl Glyph for KeyCustom {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        KeyEased { look: self.look }.paint(painter, rect, color);
        painter.circle_filled(rect.center(), Self::DOT_RADIUS, <color::BgApp as ColorToken>::VALUE);
    }
}
