use amml_ui::brand::{DvMark, DvMarkSize};
use amml_ui::font::FontFace;
use amml_ui::tokens::{ColorToken, color};
use egui::epaint::CornerRadius;
use egui::{Rect, Shape, pos2, vec2};

use crate::support::PaintCapture;

fn paint(size: DvMarkSize, rect: Rect) -> (Vec<Shape>, Rect) {
    let mut placed = Rect::NOTHING;
    let shapes = PaintCapture::shapes(|painter| placed = DvMark { size }.paint(painter, rect));
    (shapes, placed)
}

fn square(shapes: &[Shape]) -> egui::epaint::RectShape {
    match shapes.iter().find_map(|shape| if let Shape::Rect(square) = shape { Some(square.clone()) } else { None }) {
        Some(square) => square,
        None => panic!("no rect shape: {shapes:?}"),
    }
}

#[test]
fn regular_mark_is_26_px_accent_square_with_rim() {
    let rect = Rect::from_center_size(pos2(80.0, 40.0), vec2(26.0, 26.0));
    let (shapes, _) = paint(DvMarkSize::Regular, rect);
    let mark = square(&shapes);
    assert_eq!(mark.rect, rect);
    assert_eq!(mark.fill, <color::Accent as ColorToken>::VALUE);
    assert_eq!(mark.stroke.width, 1.0);
    assert_eq!(mark.stroke.color, <color::LineOverlay as ColorToken>::VALUE);
    assert_eq!(mark.corner_radius, CornerRadius::same(6));
    assert_eq!(PaintCapture::texts(&shapes), vec![("DV".to_owned(), 11.0, FontFace::Semibold.family())]);
    let text_colour =
        shapes.iter().find_map(|shape| if let Shape::Text(text) = shape { Some(text.fallback_color) } else { None });
    let section_colour = shapes.iter().find_map(|shape| match shape {
        Shape::Text(text) => text.galley.job.sections.first().map(|section| section.format.color),
        _ => None,
    });
    let on_accent = <color::OnAccent as ColorToken>::VALUE;
    assert!(text_colour == Some(on_accent) || section_colour == Some(on_accent), "the text is not on-accent");
}

#[test]
fn small_mark_is_18_px() {
    let rect = Rect::from_center_size(pos2(40.0, 40.0), vec2(18.0, 18.0));
    let (shapes, _) = paint(DvMarkSize::Small, rect);
    let mark = square(&shapes);
    assert_eq!(mark.rect, rect);
    assert_eq!(mark.corner_radius, CornerRadius::same(4));
    assert_eq!(PaintCapture::texts(&shapes), vec![("DV".to_owned(), 8.0, FontFace::Semibold.family())]);
}

#[test]
fn mark_keeps_its_side_in_a_small_rect() {
    let rect = Rect::from_center_size(pos2(50.0, 50.0), vec2(10.0, 10.0));
    let (shapes, placed) = paint(DvMarkSize::Regular, rect);
    assert_eq!(placed, Rect::from_center_size(pos2(50.0, 50.0), vec2(26.0, 26.0)));
    assert_eq!(square(&shapes).rect, placed);
}

#[test]
fn text_is_centred_in_the_mark() {
    let rect = Rect::from_center_size(pos2(50.0, 50.0), vec2(26.0, 26.0));
    let (shapes, _) = paint(DvMarkSize::Regular, rect);
    let centre = shapes.iter().find_map(|shape| match shape {
        Shape::Text(text) => Some(text.galley.rect.translate(text.pos.to_vec2()).center()),
        _ => None,
    });
    let Some(centre) = centre else { panic!("no text") };
    assert!((centre - rect.center()).length() < 0.5, "{centre:?}");
}
