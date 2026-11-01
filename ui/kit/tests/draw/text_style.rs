use amml_ui::draw::{Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use egui::Color32;

use crate::support::PaintCapture;

#[test]
fn tracked_style_sets_letter_spacing() {
    let plain = TextStyle::new(TypeSize::Caption, FontFace::Semibold);
    let mut widths = (0.0, 0.0);
    PaintCapture::shapes(|painter| {
        let a = Draw::galley(painter, "ABCD", plain, Color32::WHITE, TextFit::Line).size().x;
        let b = Draw::galley(painter, "ABCD", plain.tracked(), Color32::WHITE, TextFit::Line).size().x;
        widths = (a, b);
    });
    // 0.08 em at 12.5 points is 1 point after each of the 3 or 4 glyphs
    let extra = widths.1 - widths.0;
    assert!((2.99..=4.01).contains(&extra), "{widths:?}");
}

#[test]
fn style_uses_the_size_and_line_height_tokens() {
    let style = TextStyle::new(TypeSize::Note, FontFace::Regular);
    assert_eq!(style.font_id(), FontFace::Regular.id(13.5));
    assert_eq!(TypeSize::Note.line_height(), 18.0);
}
