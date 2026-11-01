use amml_studio_ui::glyph::NameTag;
use egui::{Rect, pos2, vec2};

use super::words;
use crate::support::PaintCapture;

#[test]
fn name_tag_is_centred_in_its_rect() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(200.0, 40.0));
    let shapes = PaintCapture::shapes(|painter| {
        NameTag { name: words("Kestrel") }.paint(painter, rect);
    });
    let bounds = PaintCapture::bounds(&shapes);
    assert!((bounds.center() - rect.center()).length() < 0.6, "{bounds:?}");
}

#[test]
fn name_tag_cuts_a_long_name() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(160.0, 40.0));
    let name = words(&"nether_nomad".repeat(17));
    let mut tag = Rect::NOTHING;
    let shapes = PaintCapture::shapes(|painter| tag = NameTag { name: name.clone() }.paint(painter, rect));
    assert!(rect.contains_rect(tag), "{tag:?}");
    for text in PaintCapture::text_rects(&shapes) {
        assert!(tag.contains_rect(text), "{text:?} leaves the tag {tag:?}");
    }
}
