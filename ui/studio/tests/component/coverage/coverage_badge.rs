use amml_studio_ui::component::coverage::CoverageBadge;
use amml_studio_ui::tokens::selector::StatusColor;
use egui::{Rect, Shape, pos2, vec2};

use crate::glyph::words;
use crate::support::PaintCapture;

#[test]
fn badge_takes_its_status_colour() {
    let badge = CoverageBadge::new(StatusColor::Missing, words("D5 MISSING"));
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(120.0, CoverageBadge::HEIGHT));
    let shapes = PaintCapture::shapes(|painter| badge.paint(painter, rect));
    let colour = StatusColor::Missing.color();
    let border = shapes.iter().any(|shape| matches!(shape, Shape::Rect(pill) if pill.stroke.color == colour));
    assert!(border, "no {colour:?} border: {shapes:?}");
    let text = shapes.iter().any(|shape| matches!(shape, Shape::Text(text) if text.fallback_color == colour));
    assert!(text, "no {colour:?} text: {shapes:?}");
}

#[test]
fn a_long_badge_label_stays_in_the_badge() {
    let badge = CoverageBadge::new(StatusColor::Planned, words(&"G11 PLANNED ".repeat(20)));
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(120.0, CoverageBadge::HEIGHT));
    let shapes = PaintCapture::shapes(|painter| badge.paint(painter, rect));
    for text in PaintCapture::text_rects(&shapes) {
        assert!(rect.contains_rect(text), "{text:?} leaves {rect:?}");
    }
}
