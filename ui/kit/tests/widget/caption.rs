use amml_ui::draw::TypeSize;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::Caption;
use egui::{Rect, pos2};

use crate::support::{PaintCapture, Sample};

struct CaptionPaint;

impl CaptionPaint {
    const HINT: &'static str = "Or drop files into mods/alaydriem-stratum/data/import";

    fn rect(caption: &Caption, width: f32) -> (Rect, Vec<egui::Shape>) {
        let mut rect = Rect::NOTHING;
        let shapes = PaintCapture::shapes(|painter| rect = caption.paint(painter, pos2(20.0, 20.0), width));
        (rect, shapes)
    }
}

#[test]
fn caption_draws_faint_caption_text() {
    let caption = Caption::new(Sample::text("Saved automatically"));
    let (rect, shapes) = CaptionPaint::rect(&caption, 400.0);
    let texts = PaintCapture::texts(&shapes);
    assert_eq!(texts.len(), 1, "one text shape: {texts:?}");
    assert_eq!(texts[0].1, TypeSize::Caption.points());
    let colour = shapes.iter().find_map(|shape| match shape {
        egui::Shape::Text(text) => Some(text.galley.job.sections[0].format.color),
        _ => None,
    });
    assert_eq!(colour, Some(color::TextFaint::VALUE));
    assert!((rect.height() - TypeSize::Caption.line_height()).abs() <= 0.5, "one line: {rect:?}");
}

#[test]
fn caption_wraps_to_the_width() {
    let caption = Caption::new(Sample::text(CaptionPaint::HINT));
    let (rect, shapes) = CaptionPaint::rect(&caption, 228.0);
    assert!(rect.height() > 1.5 * TypeSize::Caption.line_height(), "the text does not wrap: {rect:?}");
    assert!(rect.width() <= 228.0 + 0.5, "the text is wider than its space: {rect:?}");
    let texts = PaintCapture::text_rects(&shapes);
    assert!(texts.iter().all(|(_, text)| text.right() <= 20.0 + 228.0 + 0.5), "text outside the width: {texts:?}");
}
