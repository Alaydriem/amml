use amml_studio_ui::glyph::EditLockPill;
use egui::{Rect, pos2, vec2};

use super::words;
use crate::support::PaintCapture;

#[test]
fn edit_lock_pill_is_centred_in_its_rect() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(300.0, 40.0));
    let shapes = PaintCapture::shapes(|painter| {
        EditLockPill { reason: words("Read only") }.paint(painter, rect);
    });
    let bounds = PaintCapture::bounds(&shapes);
    assert!((bounds.center() - rect.center()).length() < 0.6, "{bounds:?}");
}

#[test]
fn a_long_reason_stays_in_the_pill() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(200.0, 40.0));
    let reason = words(&"Scene camera drives ".repeat(10));
    let mut pill = Rect::NOTHING;
    let shapes = PaintCapture::shapes(|painter| pill = EditLockPill { reason: reason.clone() }.paint(painter, rect));
    assert!(rect.contains_rect(pill), "{pill:?}");
    for text in PaintCapture::text_rects(&shapes) {
        assert!(pill.contains_rect(text), "{text:?} leaves the pill {pill:?}");
    }
}
