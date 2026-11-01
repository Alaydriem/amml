use amml_ui::glyph::Glyph;
use egui::{Color32, Painter, Rect, Shape, Stroke, vec2};

/// Paper: Assets, `Glyph / path map camera cone` (`3AM-0`). The camera on the path map: a dot at the centre of the
/// rect and a cone that opens the way the camera looks. `heading` is in radians, clockwise from up. The caller gives
/// the colour (`--color-playhead`, or `--color-scene-camera` while the scene camera drives).
pub struct PathMapCone {
    pub heading: f32,
}

impl PathMapCone {
    /// Paper: the cone is 26 long and 24 across its open end.
    pub const LENGTH: f32 = 26.0;
    pub const HALF_WIDTH: f32 = 12.0;
    /// The furthest any part reaches from the centre: a far corner (about 28.6) plus the outline.
    pub const REACH: f32 = 29.5;
    // Paper: a 1 px outline, a body at 25 %, a radius 4 dot
    const OUTLINE: f32 = 1.0;
    const BODY_ALPHA: u8 = 0x40;
    const DOT_RADIUS: f32 = 4.0;
}

impl Glyph for PathMapCone {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        let eye = rect.center();
        let (sin, cos) = self.heading.sin_cos();
        let ahead = vec2(sin, -cos) * Self::LENGTH;
        let across = vec2(cos, sin) * Self::HALF_WIDTH;
        let cone = vec![eye, eye + ahead - across, eye + ahead + across];
        let body = color.gamma_multiply_u8(Self::BODY_ALPHA);
        painter.add(Shape::convex_polygon(cone, body, Stroke::new(Self::OUTLINE, color)));
        painter.circle_filled(eye, Self::DOT_RADIUS, color);
    }
}
