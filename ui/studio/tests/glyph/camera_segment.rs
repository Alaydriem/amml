use amml_studio_ui::glyph::CameraSegment;
use egui::{Rect, pos2, vec2};

use super::words;
use crate::support::PaintCapture;

#[test]
fn camera_segment_stays_in_its_rect() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(220.0, CameraSegment::HEIGHT));
    for hold in [false, true] {
        let shapes = PaintCapture::shapes(|painter| CameraSegment { ease: words("Smooth"), hold }.paint(painter, rect));
        assert!(rect.contains_rect(PaintCapture::bounds(&shapes)), "{:?}", PaintCapture::bounds(&shapes));
    }
}

#[test]
fn easing_pill_is_left_out_when_the_segment_is_too_short() {
    let short = Rect::from_min_size(pos2(20.0, 20.0), vec2(12.0, CameraSegment::HEIGHT));
    let shapes =
        PaintCapture::shapes(|painter| CameraSegment { ease: words("Smooth"), hold: false }.paint(painter, short));
    assert!(PaintCapture::text_rects(&shapes).is_empty(), "a pill in a 12 px segment: {shapes:?}");
    assert!(!shapes.is_empty(), "the bar is still drawn");
}
