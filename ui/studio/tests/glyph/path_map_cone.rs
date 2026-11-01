use std::f32::consts::{FRAC_PI_2, PI};

use amml_studio_ui::glyph::PathMapCone;
use amml_ui::glyph::Glyph;
use egui::{Color32, Rect, Shape, pos2, vec2};

use crate::support::PaintCapture;

const RED: Color32 = Color32::from_rgb(0xFF, 0x4D, 0x5E);

fn rect() -> Rect {
    Rect::from_center_size(pos2(100.0, 100.0), vec2(PathMapCone::REACH * 2.0, PathMapCone::REACH * 2.0))
}

// the middle of the cone's open end: the mean of the two far corners
fn far_middle(heading: f32) -> egui::Pos2 {
    let shapes = PaintCapture::shapes(|painter| PathMapCone { heading }.paint(painter, rect(), RED));
    let points = shapes
        .iter()
        .find_map(|shape| match shape {
            Shape::Path(path) if path.closed => Some(path.points.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no cone: {shapes:?}"));
    let [_, a, b] = points[..] else { panic!("expected a triangle: {points:?}") };
    pos2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

#[test]
fn path_map_cone_stays_in_its_rect() {
    for heading in [0.0, 1.0, PI, -2.5] {
        let shapes = PaintCapture::shapes(|painter| PathMapCone { heading }.paint(painter, rect(), RED));
        assert!(rect().contains_rect(PaintCapture::bounds(&shapes)), "{heading}: {:?}", PaintCapture::bounds(&shapes));
    }
}

#[test]
fn path_map_cone_turns_with_heading() {
    let centre = rect().center();
    let up = far_middle(0.0) - centre;
    let right = far_middle(FRAC_PI_2) - centre;
    assert!(up.y < 0.0 && up.x.abs() < 0.01, "heading 0 opens up: {up:?}");
    assert!((up.length() - right.length()).abs() < 0.01);
    assert!(up.dot(right).abs() < 0.01, "not a quarter turn: {up:?} {right:?}");
    assert!(right.x > 0.0, "heading PI / 2 opens right: {right:?}");
}
