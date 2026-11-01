use amml_studio_ui::glyph::SceneCameraFrustum;
use amml_ui::glyph::Glyph;
use egui::{Color32, Rect, Shape, pos2, vec2};

use crate::support::PaintCapture;

const AMBER: Color32 = Color32::from_rgb(0xFF, 0xB5, 0x47);

#[test]
fn scene_camera_frustum_stays_in_its_rect() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(48.0, 40.0));
    for filled in [false, true] {
        let shapes = PaintCapture::shapes(|painter| SceneCameraFrustum { filled }.paint(painter, rect, AMBER));
        assert!(rect.contains_rect(PaintCapture::bounds(&shapes)), "{:?}", PaintCapture::bounds(&shapes));
    }
}

#[test]
fn only_a_filled_frustum_paints_its_body() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(48.0, 40.0));
    let body = |filled: bool| {
        let shapes = PaintCapture::shapes(|painter| SceneCameraFrustum { filled }.paint(painter, rect, AMBER));
        shapes.iter().any(|shape| matches!(shape, Shape::Path(path) if path.closed && path.fill != Color32::TRANSPARENT && path.fill != AMBER))
    };
    assert!(body(true));
    assert!(!body(false));
}
