use amml_studio_ui::glyph::BookmarkFlag;
use amml_ui::glyph::Glyph;
use egui::{Color32, Rect, pos2, vec2};

use super::words;
use crate::support::PaintCapture;

const MARKER: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF2);

#[test]
fn bookmark_flag_stays_in_its_rect() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(BookmarkFlag::WIDTH, 70.0));
    let shapes = PaintCapture::shapes(|painter| BookmarkFlag { label: words("portal") }.paint(painter, rect, MARKER));
    assert!(rect.contains_rect(PaintCapture::bounds(&shapes)), "{:?}", PaintCapture::bounds(&shapes));
}

#[test]
fn a_long_bookmark_name_stays_in_the_flag() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(BookmarkFlag::WIDTH, 70.0));
    let label = words(&"portal ".repeat(40));
    let shapes = PaintCapture::shapes(|painter| BookmarkFlag { label: label.clone() }.paint(painter, rect, MARKER));
    let flag = Rect::from_min_size(rect.min, vec2(BookmarkFlag::WIDTH, BookmarkFlag::HEIGHT));
    for text in PaintCapture::text_rects(&shapes) {
        assert!(flag.contains_rect(text), "{text:?} leaves the flag {flag:?}");
    }
}
