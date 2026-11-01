use amml_studio_ui::component::easing::{CurvePlot, CurvePlotView};
use egui::{Color32, Rect, Shape, pos2, vec2};

use crate::support::PaintCapture;

const FOV: Color32 = Color32::from_rgb(0x2E, 0xC4, 0xD6);

fn rect() -> Rect {
    Rect::from_min_size(pos2(20.0, 20.0), vec2(288.0, 160.0))
}

fn paint(view: &CurvePlotView) -> Vec<Shape> {
    PaintCapture::shapes(|painter| CurvePlot::new(view).paint(painter, rect()))
}

// the points of the curve: the only open line in the track colour, 2 wide
fn curve(shapes: &[Shape]) -> Option<Vec<egui::Pos2>> {
    shapes.iter().find_map(|shape| match shape {
        Shape::Path(path) if !path.closed && path.points.len() > 2 && path.stroke.width == CurvePlot::CURVE_WIDTH => {
            Some(path.points.clone())
        }
        _ => None,
    })
}

#[test]
fn curve_plot_maps_zero_and_one_to_the_box() {
    let samples = [0.0, 0.25, 0.5, 0.75, 1.0];
    let shapes = paint(&CurvePlotView { samples: &samples, color: FOV, handles: None });
    let points = curve(&shapes).unwrap_or_else(|| panic!("no curve: {shapes:?}"));
    let plot = CurvePlot::plot_box(rect());
    let (first, last) = (points[0], points[points.len() - 1]);
    assert!((first - plot.left_bottom()).length() < 0.01, "{first:?} is not {:?}", plot.left_bottom());
    assert!((last - plot.right_top()).length() < 0.01, "{last:?} is not {:?}", plot.right_top());
}

#[test]
fn curve_plot_with_no_samples() {
    let shapes = paint(&CurvePlotView { samples: &[], color: FOV, handles: None });
    assert!(curve(&shapes).is_none(), "a curve with no samples: {shapes:?}");
    assert!(!shapes.is_empty(), "the frame is still drawn");
}

#[test]
fn curve_plot_handles_draw_two_lines() {
    let samples = [0.0, 0.5, 1.0];
    let handles = Some([pos2(0.4, 0.0), pos2(0.6, 1.0)]);
    let without = paint(&CurvePlotView { samples: &samples, color: FOV, handles: None });
    let with = paint(&CurvePlotView { samples: &samples, color: FOV, handles });
    let knobs = |shapes: &[Shape]| shapes.iter().filter(|shape| matches!(shape, Shape::Circle(_))).count();
    let lines = |shapes: &[Shape]| {
        shapes
            .iter()
            .filter(|shape| matches!(shape, Shape::LineSegment { stroke, .. } if stroke.color != FOV && stroke.width < CurvePlot::CURVE_WIDTH))
            .count()
    };
    assert_eq!(knobs(&with) - knobs(&without), 2);
    assert_eq!(lines(&with) - lines(&without), 2);
}

#[test]
fn curve_plot_stays_in_its_rect_when_a_sample_overshoots() {
    let samples = [0.0, 1.4, -0.3, 1.0];
    let shapes =
        paint(&CurvePlotView { samples: &samples, color: FOV, handles: Some([pos2(0.3, 1.6), pos2(0.7, -0.6)]) });
    let bounds = PaintCapture::bounds(&shapes);
    assert!(rect().contains_rect(bounds), "{bounds:?}");
}
