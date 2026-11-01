use amml_ui::draw::{Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use egui::Color32;

use crate::support::PaintCapture;

#[test]
fn truncate_keeps_one_row_inside_the_width() {
    let long = "a very long scene name ".repeat(9);
    let style = TextStyle::new(TypeSize::Note, FontFace::Regular);
    let mut checked = false;
    PaintCapture::shapes(|painter| {
        let galley = Draw::galley(painter, &long, style, Color32::WHITE, TextFit::Truncate(80.0));
        assert_eq!(galley.rows.len(), 1);
        assert!(galley.size().x <= 80.0 + 0.5, "{}", galley.size().x);
        let last = galley.rows[0].glyphs.last().map(|glyph| glyph.chr);
        assert_eq!(last, Some('…'));
        checked = true;
    });
    assert!(checked);
}

#[test]
fn wrap_breaks_into_rows() {
    let long = "word ".repeat(40);
    let style = TextStyle::new(TypeSize::Note, FontFace::Regular);
    let mut rows = 0;
    PaintCapture::shapes(|painter| {
        rows = Draw::galley(painter, &long, style, Color32::WHITE, TextFit::Wrap(100.0)).rows.len();
    });
    assert!(rows > 1);
}
