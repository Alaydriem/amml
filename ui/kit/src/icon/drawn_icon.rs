use egui::{Color32, Painter, Rect, vec2};

use super::IconSize;
use crate::glyph::SnapMagnet;

/// An icon that the icon font does not have, so it is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawnIcon {
    Snap,
}

impl DrawnIcon {
    /// Paints into a square of the icon size, centred on `rect`.
    pub fn paint(self, painter: &Painter, rect: Rect, size: IconSize, color: Color32) {
        let square = Rect::from_center_size(rect.center(), vec2(size.points(), size.points()));
        match self {
            Self::Snap => SnapMagnet::paint(painter, square, color),
        }
    }
}
