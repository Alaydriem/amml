use amml_ui::glyph::{Spinner, SpinnerSize};
use egui::{Color32, Pos2, Rect, Shape, pos2, vec2};

use crate::support::PaintCapture;

const CENTRE: Pos2 = pos2(60.0, 60.0);

fn paint(size: SpinnerSize, turn: f32, color: Color32) -> Vec<Shape> {
    let rect = Rect::from_center_size(CENTRE, vec2(40.0, 40.0));
    PaintCapture::shapes(|painter| Spinner { size, turn }.paint(painter, rect, color))
}

fn arc_start(size: SpinnerSize, turn: f32) -> Pos2 {
    match paint(size, turn, Color32::WHITE).get(1) {
        Some(Shape::Path(path)) => path.points[0],
        other => panic!("the second shape is not the arc: {other:?}"),
    }
}

#[test]
fn spinner_paints_track_then_arc() {
    let shapes = paint(SpinnerSize::Regular, 0.0, Color32::WHITE);
    let Some(Shape::Circle(track)) = shapes.first() else { panic!("the first shape is not the track: {shapes:?}") };
    assert_eq!(track.center, CENTRE);
    assert_eq!(track.radius, 8.0);
    assert_eq!(track.stroke.width, 2.0);
    assert_eq!(track.stroke.color.a(), 77);
    let start = arc_start(SpinnerSize::Regular, 0.0);
    assert!((start - pos2(60.0, 52.0)).length() < 0.01, "the arc starts at {start:?}, not at 12 o'clock");
    let Some(Shape::Path(arc)) = shapes.get(1) else { panic!("no arc") };
    let end = arc.points[arc.points.len() - 1];
    assert!((end - pos2(68.0, 60.0)).length() < 0.01, "the arc ends at {end:?}, not at 3 o'clock");
}

#[test]
fn small_spinner_uses_small_geometry() {
    let shapes = paint(SpinnerSize::Small, 0.0, Color32::WHITE);
    let Some(Shape::Circle(track)) = shapes.first() else { panic!("no track") };
    assert_eq!(track.radius, 5.5);
    assert_eq!(track.stroke.width, 1.75);
    assert_eq!(SpinnerSize::Small.side(), 14.0);
    assert_eq!(SpinnerSize::Regular.side(), 20.0);
}

#[test]
fn turn_wraps() {
    for (turn, same) in [(3.25, 0.25), (-0.25, 0.75)] {
        let (a, b) = (arc_start(SpinnerSize::Regular, turn), arc_start(SpinnerSize::Regular, same));
        assert!((a - b).length() < 0.01, "turn {turn} starts at {a:?}, turn {same} at {b:?}");
    }
}

#[test]
fn transparent_colour_paints_without_panic() {
    assert_eq!(paint(SpinnerSize::Small, 0.5, Color32::TRANSPARENT).len(), 2);
}
