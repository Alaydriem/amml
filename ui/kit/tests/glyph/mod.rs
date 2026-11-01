mod key;
mod snap_magnet;
mod spinner;
mod swatch_dot;

use amml_ui::glyph::{Glyph, SnapMagnet, Spinner, SpinnerSize, SwatchDot};
use egui::{Color32, Rect, Shape, pos2};

use crate::support::PaintCapture;

#[test]
fn glyph_trait_paints_the_same_shapes() {
    let rect = Rect::from_min_max(pos2(40.0, 40.0), pos2(60.0, 60.0));
    let spinner = Spinner { size: SpinnerSize::Small, turn: 0.25 };
    let cases: [(&dyn Glyph, Vec<Shape>); 3] = [
        (&SwatchDot, PaintCapture::shapes(|p| SwatchDot::paint(p, rect, Color32::RED))),
        (&SnapMagnet, PaintCapture::shapes(|p| SnapMagnet::paint(p, rect, Color32::RED))),
        (&spinner, PaintCapture::shapes(|p| spinner.paint(p, rect, Color32::RED))),
    ];
    for (glyph, direct) in cases {
        let through_trait = PaintCapture::shapes(|p| glyph.paint(p, rect, Color32::RED));
        assert_eq!(through_trait, direct);
    }
}
