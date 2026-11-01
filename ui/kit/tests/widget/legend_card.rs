use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{LegendCard, LegendRow};
use egui::{Color32, Rect, pos2, vec2};

use crate::support::{PaintCapture, Sample, ShapeText};

struct CardPaint;

impl CardPaint {
    const CYAN: Color32 = Color32::from_rgb(0x2E, 0xC4, 0xD6);
    const AMBER: Color32 = Color32::from_rgb(0xFF, 0xB5, 0x47);
    const RED: Color32 = Color32::from_rgb(0xFF, 0x4D, 0x5E);

    fn card(rows: usize) -> LegendCard {
        let rows = [(Self::CYAN, "Key frustum"), (Self::AMBER, "Selected key"), (Self::RED, "Scene camera")]
            .into_iter()
            .take(rows)
            .map(|(colour, label)| LegendRow::new(colour, Sample::text(label)))
            .collect();
        LegendCard::new(Sample::text("In-world camera path"))
            .note(Sample::text("Only the active scene's path."))
            .rows(rows)
    }

    fn at(card: &LegendCard, width: f32) -> (Rect, Vec<egui::Shape>) {
        let mut rect = Rect::NOTHING;
        let shapes = PaintCapture::shapes(|painter| rect = card.paint(painter, pos2(20.0, 20.0), width));
        (rect, shapes)
    }
}

#[test]
fn card_is_as_wide_as_asked_and_grows_with_its_rows() {
    let (three, _) = CardPaint::at(&CardPaint::card(3), 476.0);
    let (one, _) = CardPaint::at(&CardPaint::card(1), 476.0);
    assert_eq!(three.width(), 476.0);
    assert!(three.height() > one.height(), "{three:?} {one:?}");
}

#[test]
fn text_comes_from_the_caller_in_reading_order() {
    let (_, shapes) = CardPaint::at(&CardPaint::card(3), 476.0);
    assert_eq!(
        ShapeText::all(&shapes),
        ["In-world camera path", "Only the active scene's path.", "Key frustum", "Selected key", "Scene camera"]
    );
}

#[test]
fn each_row_has_a_square_swatch_in_its_own_colour() {
    let (card, shapes) = CardPaint::at(&CardPaint::card(3), 476.0);
    let rects = PaintCapture::rects(&shapes);
    let swatches: Vec<_> = rects.iter().filter(|r| r.rect.size() == vec2(12.0, 12.0)).collect();
    let colours: Vec<Color32> = swatches.iter().map(|r| r.fill).collect();
    assert_eq!(colours, [CardPaint::CYAN, CardPaint::AMBER, CardPaint::RED]);
    for swatch in swatches {
        assert!(card.contains_rect(swatch.rect));
        // a rounded square, not the round dot
        assert!(swatch.corner_radius.nw < 6, "{:?}", swatch.corner_radius);
        assert!(swatch.corner_radius.nw > 0);
    }
}

#[test]
fn card_sits_on_a_faded_app_background_with_the_overlay_line() {
    let (card, shapes) = CardPaint::at(&CardPaint::card(3), 476.0);
    let rects = PaintCapture::rects(&shapes);
    let faded = color::BgApp::VALUE.gamma_multiply(0.84);
    assert!(rects.iter().any(|r| r.rect == card && r.fill == faded), "no faded fill");
    assert!(rects.iter().any(|r| r.rect == card && r.stroke.color == color::LineOverlay::VALUE), "no overlay line");
}

#[test]
fn a_card_without_a_note_is_shorter() {
    let with_note = CardPaint::card(2);
    let without = LegendCard::new(Sample::text("In-world camera path")).rows(vec![
        LegendRow::new(
            CardPaint::CYAN,
            Sample::text("Key frustum")
        );
        2
    ]);
    let (tall, _) = CardPaint::at(&with_note, 476.0);
    let (short, shapes) = CardPaint::at(&without, 476.0);
    assert!(short.height() < tall.height());
    assert_eq!(ShapeText::all(&shapes), ["In-world camera path", "Key frustum", "Key frustum"]);
}
