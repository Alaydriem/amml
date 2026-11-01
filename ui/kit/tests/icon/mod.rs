use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use egui::{Color32, Rect, Shape, pos2, vec2};

use crate::support::PaintCapture;

fn paint(icon: Icon, rect: Rect, size: IconSize) -> Vec<Shape> {
    PaintCapture::shapes(|painter| icon.paint(painter, rect, size, Color32::WHITE))
}

fn tile() -> Rect {
    Rect::from_center_size(pos2(100.0, 100.0), vec2(44.0, 44.0))
}

#[test]
fn font_icon_paints_its_code_point() {
    let shapes = paint(Icon::Add, tile(), IconSize::Regular);
    assert_eq!(PaintCapture::texts(&shapes), vec![("\u{E710}".to_owned(), 16.0, FontFace::Icons.family())]);
}

#[test]
fn small_icon_uses_the_small_size() {
    let shapes = paint(Icon::Close, tile(), IconSize::Small);
    assert_eq!(PaintCapture::texts(&shapes), vec![("\u{E8BB}".to_owned(), 12.0, FontFace::Icons.family())]);
}

#[test]
fn icon_is_centred_whatever_the_rect() {
    let centre = pos2(100.0, 100.0);
    for rect in [tile(), Rect::from_center_size(centre, vec2(0.0, 0.0))] {
        let shapes = paint(Icon::Search, rect, IconSize::Regular);
        let text = shapes.iter().find_map(|shape| match shape {
            Shape::Text(text) => Some(text.galley.rect.translate(text.pos.to_vec2()).center()),
            _ => None,
        });
        let Some(text_centre) = text else { panic!("no text shape for {rect:?}") };
        assert!((text_centre - centre).length() < 0.5, "{text_centre:?} is not at {centre:?}");
    }
}

#[test]
fn every_icon_paints_with_built_in_fonts() {
    for icon in Icon::ALL {
        assert!(!paint(icon, tile(), IconSize::Regular).is_empty(), "{icon:?} painted nothing");
    }
}

#[test]
fn all_icons_are_distinct() {
    for (at, icon) in Icon::ALL.iter().enumerate() {
        for other in &Icon::ALL[at + 1..] {
            assert_ne!(icon.shape(), other.shape(), "{icon:?} and {other:?} draw the same thing");
        }
    }
}

#[test]
fn snap_is_drawn_not_text() {
    let shapes = paint(Icon::Snap, tile(), IconSize::Regular);
    assert!(PaintCapture::texts(&shapes).is_empty());
    assert!(shapes.iter().any(|shape| matches!(shape, Shape::Path(_) | Shape::LineSegment { .. })));
}

#[test]
fn drawn_icon_is_centred_and_sized_by_icon_size() {
    let shapes = paint(Icon::Snap, tile(), IconSize::Small);
    let bounds = shapes.iter().fold(Rect::NOTHING, |bounds, shape| bounds.union(shape.visual_bounding_rect()));
    assert!((bounds.center() - pos2(100.0, 100.0)).length() < 1.0, "{bounds:?}");
    assert!(bounds.width() <= 13.0 && bounds.height() <= 13.0, "{bounds:?} is larger than a small icon");
}
