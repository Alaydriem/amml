use amml_studio_ui::glyph::Playhead;
use amml_ui::glyph::Glyph;
use egui::{Color32, Rect, Shape, pos2, vec2};

use super::words;
use crate::support::PaintCapture;

const RED: Color32 = Color32::from_rgb(0xFF, 0x4D, 0x5E);

#[test]
fn playhead_stays_in_its_rect() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(Playhead::HEAD_WIDTH, 80.0));
    let shapes = PaintCapture::shapes(|painter| Playhead { label: words("32.95") }.paint(painter, rect, RED));
    assert!(rect.contains_rect(PaintCapture::bounds(&shapes)), "{:?}", PaintCapture::bounds(&shapes));
}

#[test]
fn playhead_line_reaches_the_bottom_of_the_rect() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(Playhead::HEAD_WIDTH, 200.0));
    let shapes = PaintCapture::shapes(|painter| Playhead { label: words("32.95") }.paint(painter, rect, RED));
    let reaches = shapes.iter().any(|shape| match shape {
        Shape::Rect(line) => line.fill == RED && (line.rect.bottom() - rect.bottom()).abs() < 0.01,
        _ => false,
    });
    assert!(reaches, "no line to the bottom: {shapes:?}");
}

#[test]
fn a_long_playhead_label_stays_in_the_head() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(Playhead::HEAD_WIDTH, 80.0));
    let label = words(&"9".repeat(200));
    let shapes = PaintCapture::shapes(|painter| Playhead { label: label.clone() }.paint(painter, rect, RED));
    let head = Rect::from_min_size(rect.min, vec2(Playhead::HEAD_WIDTH, Playhead::HEAD_HEIGHT));
    for text in PaintCapture::text_rects(&shapes) {
        assert!(head.contains_rect(text), "{text:?} leaves the head {head:?}");
    }
}
