use amml_ui::draw::TypeSize;
use amml_ui::tokens::{SpacingToken, spacing};
use amml_ui::widget::{ValueRow, ValueRowSize};
use egui::{Rect, pos2};

use crate::support::{ClickProbe, PaintCapture, Sample};

struct RowPaint;

impl RowPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(380.0, 48.0));

    fn texts(row: &ValueRow, rect: Rect) -> Vec<(String, Rect)> {
        PaintCapture::text_rects(&PaintCapture::shapes(|painter| row.paint(painter, rect)))
    }

    fn rect_of(texts: &[(String, Rect)], text: &str) -> Rect {
        match texts.iter().find(|(painted, _)| painted == text) {
            Some((_, rect)) => *rect,
            None => panic!("{text} was not painted: {texts:?}"),
        }
    }
}

#[test]
fn label_sits_left_and_value_right() {
    let row = ValueRow::new(Sample::text("Length"), Sample::text("02:14:08"));
    let texts = RowPaint::texts(&row, RowPaint::RECT);
    assert_eq!(RowPaint::rect_of(&texts, "Length").left(), RowPaint::RECT.left());
    assert_eq!(RowPaint::rect_of(&texts, "02:14:08").right(), RowPaint::RECT.right());
}

#[test]
fn only_the_given_text_is_painted() {
    let row = ValueRow::new(Sample::text("Output"), Sample::text("6.20 s"));
    let painted: Vec<String> = RowPaint::texts(&row, RowPaint::RECT).into_iter().map(|(text, _)| text).collect();
    assert_eq!(painted, ["Output", "6.20 s"]);
}

#[test]
fn large_row_is_row_high_and_compact_is_control_small_high() {
    for (size, height) in [
        (ValueRowSize::Large, <spacing::Row as SpacingToken>::VALUE),
        (ValueRowSize::Compact, <spacing::ControlSmall as SpacingToken>::VALUE),
    ] {
        let mut harness = ClickProbe::harness(move |ui| {
            ValueRow::new(Sample::text("Length"), Sample::text("02:14:08")).size(size).show(ui)
        });
        harness.run();
        assert_eq!(harness.state().rect.height(), height, "{size:?}");
    }
}

#[test]
fn value_is_caption_size_and_words_are_note_size() {
    for (words, expected) in [(false, TypeSize::Caption), (true, TypeSize::Note)] {
        let mut row = ValueRow::new(Sample::text("Voice"), Sample::text("BVC session linked"));
        if words {
            row = row.words();
        }
        let shapes = PaintCapture::shapes(|painter| row.paint(painter, RowPaint::RECT));
        let size = PaintCapture::texts(&shapes).into_iter().find(|(text, ..)| text == "BVC session linked");
        assert_eq!(size.map(|(_, size, _)| size), Some(expected.points()), "words: {words}");
    }
}

#[test]
fn long_value_does_not_cover_the_label() {
    let row = ValueRow::new(Sample::text("Voice"), Sample::text("BVC session linked to the proximity channel"));
    let narrow = Rect::from_min_max(pos2(20.0, 20.0), pos2(260.0, 48.0));
    let texts = RowPaint::texts(&row, narrow);
    let label = RowPaint::rect_of(&texts, "Voice");
    let value = texts.iter().find(|(text, _)| text.starts_with("BVC")).map(|(_, rect)| *rect);
    let Some(value) = value else { panic!("no value painted") };
    assert!(label.right() <= value.left(), "label {label:?} runs into value {value:?}");
    assert!(value.right() <= narrow.right() + 0.5, "value {value:?} leaves the row {narrow:?}");
}
