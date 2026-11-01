use amml_ui::glyph::SnapMagnet;
use egui::{Color32, Pos2, Rect, Shape, pos2, vec2};

use crate::support::PaintCapture;

// every stroked point and its stroke width
fn strokes(rect: Rect) -> Vec<(Pos2, f32)> {
    let shapes = PaintCapture::shapes(|painter| SnapMagnet::paint(painter, rect, Color32::WHITE));
    let mut out = Vec::new();
    for shape in shapes {
        match shape {
            Shape::Path(path) => out.extend(path.points.iter().map(|point| (*point, path.stroke.width))),
            Shape::LineSegment { points, stroke } => out.extend(points.iter().map(|point| (*point, stroke.width))),
            _ => {}
        }
    }
    out
}

#[test]
fn snap_fits_the_icon_square() {
    let rect = Rect::from_min_size(pos2(100.0, 100.0), vec2(16.0, 16.0));
    let points = strokes(rect);
    assert!(!points.is_empty());
    for (point, width) in points {
        assert!(rect.expand(width / 2.0).contains(point), "{point:?} is outside {rect:?}");
    }
}

#[test]
fn snap_stroke_scales_with_size() {
    let width = |side: f32| strokes(Rect::from_min_size(Pos2::ZERO, vec2(side, side)))[0].1;
    assert!((width(24.0) - 1.6).abs() < 1e-4);
    assert!((width(12.0) - 0.8).abs() < 1e-4);
}

#[test]
fn snap_uses_the_shorter_side() {
    let wide = Rect::from_center_size(pos2(50.0, 50.0), vec2(48.0, 24.0));
    for (point, _) in strokes(wide) {
        assert!((point.x - 50.0).abs() <= 12.5, "{point:?} spreads past the 24 px square");
    }
}
