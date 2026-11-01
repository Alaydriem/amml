use egui::{Color32, Painter, Rect, Shape, Stroke, pos2};

use crate::glyph::Glyph;

/// A stand-in for the studio key diamond (Paper: Text button sheet, "Add camera key"). The real
/// glyph is an amml-studio-ui type (Plan 07); amml-ui cannot depend on it.
pub struct SampleDiamond;

impl Glyph for SampleDiamond {
    fn paint(&self, painter: &Painter, rect: Rect, color: Color32) {
        let c = rect.center();
        let r = rect.width().min(rect.height()) / 2.0;
        let points = vec![pos2(c.x, c.y - r), pos2(c.x + r, c.y), pos2(c.x, c.y + r), pos2(c.x - r, c.y)];
        painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
    }
}
