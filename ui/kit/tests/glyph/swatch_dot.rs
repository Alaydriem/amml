use amml_ui::glyph::SwatchDot;
use egui::{Color32, Rect, Shape, pos2, vec2};

use crate::support::PaintCapture;

#[test]
fn swatch_is_a_filled_10_px_circle_at_the_centre() {
    let colour = Color32::from_rgb(0xB5, 0xE8, 0x53);
    let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(28.0, 20.0));
    let shapes = PaintCapture::shapes(|painter| SwatchDot::paint(painter, rect, colour));
    let [Shape::Circle(dot)] = shapes.as_slice() else { panic!("expected one circle: {shapes:?}") };
    assert_eq!(dot.center, rect.center());
    assert_eq!(dot.radius, 5.0);
    assert_eq!(dot.fill, colour);
}
