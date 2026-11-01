use amml_ui::glyph::{Glyph, KeyAuto, KeyCustom, KeyEased, KeyHold, KeyLook};
use egui::{Color32, Rect, Shape, pos2, vec2};

use crate::support::PaintCapture;

const TRACK: Color32 = Color32::from_rgb(0xFF, 0xB5, 0x47);

fn every_key(look: KeyLook) -> [Box<dyn Glyph>; 4] {
    [Box::new(KeyHold { look }), Box::new(KeyAuto { look }), Box::new(KeyEased { look }), Box::new(KeyCustom { look })]
}

fn bounds(shapes: &[Shape]) -> Rect {
    shapes.iter().fold(Rect::NOTHING, |all, shape| all.union(shape.visual_bounding_rect()))
}

#[test]
fn rest_key_glyphs_stay_in_a_20_px_rect() {
    let rect = Rect::from_min_size(pos2(40.0, 40.0), vec2(20.0, 20.0));
    for glyph in every_key(KeyLook::Rest) {
        let shapes = PaintCapture::shapes(|painter| glyph.paint(painter, rect, TRACK));
        assert!(rect.contains_rect(bounds(&shapes)), "{:?} leaves {rect:?}", bounds(&shapes));
    }
}

#[test]
fn selected_key_glyphs_stay_in_their_halo() {
    // the halo has radius 11, so a selected key needs a 22 px rect
    let rect = Rect::from_center_size(pos2(50.0, 50.0), vec2(22.0, 22.0));
    for glyph in every_key(KeyLook::Selected) {
        let shapes = PaintCapture::shapes(|painter| glyph.paint(painter, rect, TRACK));
        assert!(rect.contains_rect(bounds(&shapes)), "{:?} leaves {rect:?}", bounds(&shapes));
    }
}

#[test]
fn key_glyphs_are_centred_on_the_rect() {
    let rect = Rect::from_min_size(pos2(10.0, 30.0), vec2(40.0, 20.0));
    for look in [KeyLook::Rest, KeyLook::Selected] {
        for glyph in every_key(look) {
            let shapes = PaintCapture::shapes(|painter| glyph.paint(painter, rect, TRACK));
            let centre = bounds(&shapes).center();
            assert!((centre - rect.center()).length() < 0.01, "{centre:?} is not {:?}", rect.center());
        }
    }
}

#[test]
fn key_glyphs_allocate_nothing() {
    let rect = Rect::from_center_size(pos2(50.0, 50.0), vec2(22.0, 22.0));
    for look in [KeyLook::Rest, KeyLook::Selected] {
        let glyphs: [&dyn Glyph; 2] = [&KeyEased { look }, &KeyHold { look }];
        for glyph in glyphs {
            let shapes = PaintCapture::shapes(|painter| glyph.paint(painter, rect, TRACK));
            assert!(
                shapes.iter().all(|shape| !matches!(shape, Shape::Path(_) | Shape::Mesh(_))),
                "a point list or mesh: {shapes:?}"
            );
        }
    }
}

#[test]
fn key_custom_is_eased_plus_dot() {
    let rect = Rect::from_center_size(pos2(50.0, 50.0), vec2(22.0, 22.0));
    for look in [KeyLook::Rest, KeyLook::Selected] {
        let eased = PaintCapture::shapes(|painter| KeyEased { look }.paint(painter, rect, TRACK));
        let custom = PaintCapture::shapes(|painter| KeyCustom { look }.paint(painter, rect, TRACK));
        assert_eq!(custom[..eased.len()], eased[..]);
        let [Shape::Circle(dot)] = &custom[eased.len()..] else {
            panic!("expected one dot after the diamond: {custom:?}")
        };
        assert_eq!(dot.center, rect.center());
    }
}

#[test]
fn a_selected_key_is_larger_than_a_rest_key_and_has_a_halo_in_the_track_colour() {
    let rect = Rect::from_center_size(pos2(50.0, 50.0), vec2(22.0, 22.0));
    for (rest, selected) in every_key(KeyLook::Rest).into_iter().zip(every_key(KeyLook::Selected)) {
        let rest = PaintCapture::shapes(|painter| rest.paint(painter, rect, TRACK));
        let selected = PaintCapture::shapes(|painter| selected.paint(painter, rect, TRACK));
        let Shape::Circle(halo) = &selected[0] else { panic!("expected the halo first: {selected:?}") };
        let hue = halo.fill.to_opaque();
        let near = |a: u8, b: u8| a.abs_diff(b) <= 2;
        assert!(near(hue.r(), TRACK.r()) && near(hue.g(), TRACK.g()) && near(hue.b(), TRACK.b()), "{hue:?}");
        assert!(halo.fill.a() < 255);
        assert!(bounds(&selected[1..]).width() > bounds(&rest).width());
    }
}
