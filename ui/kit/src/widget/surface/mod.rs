mod surface_level;

pub use surface_level::SurfaceLevel;

use egui::{CornerRadius, Frame, Painter, Rect, Stroke};

use crate::draw::{Corner, Draw};
use crate::tokens::rule::Elevation;

/// Paper: Widgets, `Widget / Card frame + surfaces`. A surface of one level, flat or floating. It
/// is a frame builder: the caller adds the content into the frame.
pub struct Surface;

impl Surface {
    /// The frame: the level's fill, border, corner and padding, and the elevation's shadow.
    pub fn frame(level: SurfaceLevel, elevation: Elevation) -> Frame {
        let stroke = level.border(elevation).map_or(Stroke::NONE, Draw::border);
        // a full corner is half the height, which a frame learns only from its content
        let radius = match level.corner() {
            Corner::Full => CornerRadius::same(u8::MAX),
            corner => corner.radius(Rect::NOTHING),
        };
        Frame::new()
            .fill(level.fill())
            .stroke(stroke)
            .corner_radius(radius)
            .inner_margin(level.margin())
            .shadow(elevation.shadow())
    }

    /// Paints the surface into `rect`, for a caller that places it.
    pub fn paint(painter: &Painter, rect: Rect, level: SurfaceLevel, elevation: Elevation) {
        Draw::surface(painter, rect, level.corner(), level.fill(), level.border(elevation), elevation);
    }
}
