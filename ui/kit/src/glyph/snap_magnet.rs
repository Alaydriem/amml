use std::f32::consts::PI;

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

/// Paper: Assets, `Icon / SNAP (drawn)`. A horseshoe magnet, drawn because the icon font has none.
pub struct SnapMagnet;

impl SnapMagnet {
    /// The side of the Paper view box that the points below are in.
    pub const VIEW_BOX: f32 = 24.0;
    /// The stroke width in the view box.
    pub const STROKE: f32 = 1.6;
    const ARC_SEGMENTS: usize = 12;

    /// Paints the magnet into the largest square that fits in `rect`, centred.
    pub fn paint(painter: &Painter, rect: Rect, color: Color32) {
        let side = rect.width().min(rect.height());
        let origin = rect.center() - vec2(side, side) / 2.0;
        let scale = side / Self::VIEW_BOX;
        let at = |x: f32, y: f32| origin + vec2(x, y) * scale;
        let stroke = Stroke::new(Self::STROKE * scale, color);

        let mut outer = vec![at(6.0, 3.5)];
        outer.extend(Self::lower_arc(&at, 6.0));
        outer.push(at(18.0, 3.5));
        painter.add(Shape::line(outer, stroke));

        let mut inner = vec![at(6.0, 3.5), at(9.5, 3.5)];
        inner.extend(Self::lower_arc(&at, 2.5));
        inner.extend([at(14.5, 3.5), at(18.0, 3.5)]);
        painter.add(Shape::line(inner, stroke));

        painter.line_segment([at(6.0, 7.5), at(9.5, 7.5)], stroke);
        painter.line_segment([at(14.5, 7.5), at(18.0, 7.5)], stroke);
    }

    // the lower half circle around (12, 12), from the left end to the right end
    fn lower_arc(at: &impl Fn(f32, f32) -> Pos2, radius: f32) -> Vec<Pos2> {
        (0..=Self::ARC_SEGMENTS)
            .map(|step| {
                let angle = PI * (1.0 - step as f32 / Self::ARC_SEGMENTS as f32);
                let point = pos2(12.0 + radius * angle.cos(), 12.0 + radius * angle.sin());
                at(point.x, point.y)
            })
            .collect()
    }
}
