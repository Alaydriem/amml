use amml_i18n::text::UserText;
use amml_ui::draw::Corner;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{Avatar, AvatarImage, AvatarSize};
use egui::{Color32, Rect, Shape, TextureId, pos2, vec2};

use crate::support::PaintCapture;

struct AvatarPaint;

impl AvatarPaint {
    fn shapes(name: &str, size: AvatarSize, off: bool) -> (Rect, Vec<Shape>) {
        let side = size.side();
        let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(side, side));
        let avatar = Avatar::new(&UserText::from_model(name.to_owned()), size).off(off);
        (rect, PaintCapture::shapes(|painter| avatar.paint(painter, rect)))
    }

    fn image_shapes(texture: TextureId, off: bool) -> (Rect, Vec<Shape>) {
        let side = AvatarSize::Regular.side();
        let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(side, side));
        let image = AvatarImage::new(texture, vec2(16.0, 16.0));
        let avatar =
            Avatar::new(&UserText::from_model("Freecam".to_owned()), AvatarSize::Regular).image(Some(image)).off(off);
        (rect, PaintCapture::shapes(|painter| avatar.paint(painter, rect)))
    }

    // (text, colour) of every text shape
    fn letters(shapes: &[Shape]) -> Vec<(String, Color32)> {
        shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Text(text) => Some((text.galley.text().to_owned(), text.galley.job.sections[0].format.color)),
                _ => None,
            })
            .collect()
    }
}

#[test]
fn letter_is_first_char_upper() {
    for (name, letter) in [("freecam", "F"), ("", "?"), ("ßeta", "SS")] {
        let (_, shapes) = AvatarPaint::shapes(name, AvatarSize::Regular, false);
        let letters: Vec<_> = AvatarPaint::letters(&shapes).into_iter().map(|(text, _)| text).collect();
        assert_eq!(letters, vec![letter.to_owned()], "{name:?}");
    }
}

#[test]
fn off_uses_faint_letter() {
    let (_, on) = AvatarPaint::shapes("Minimap", AvatarSize::Regular, false);
    assert_eq!(AvatarPaint::letters(&on)[0].1, color::TextDim::VALUE);
    let (rect, off) = AvatarPaint::shapes("World Edit", AvatarSize::Regular, true);
    assert_eq!(AvatarPaint::letters(&off)[0].1, color::TextFaint::VALUE);
    // only the letter changes
    let tile = PaintCapture::rects(&off).into_iter().find(|r| r.rect == rect);
    assert_eq!(tile.map(|r| r.fill), Some(color::BgField::VALUE));
}

#[test]
fn large_uses_panel_corner() {
    for (size, corner) in [(AvatarSize::Large, Corner::Panel), (AvatarSize::Regular, Corner::Control)] {
        let (rect, shapes) = AvatarPaint::shapes("Freecam", size, false);
        let tile = PaintCapture::rects(&shapes).into_iter().find(|r| r.rect == rect && r.fill == color::BgField::VALUE);
        let Some(tile) = tile else { panic!("{size:?}: no tile on the rect") };
        assert_eq!(tile.corner_radius, corner.radius(rect), "{size:?}");
    }
}

#[test]
fn cover_uv_crops_wide_and_tall_about_the_centre() {
    let texture = TextureId::Managed(7);
    assert_eq!(
        AvatarImage::new(texture, vec2(200.0, 100.0)).cover_uv(),
        Rect::from_min_max(pos2(0.25, 0.0), pos2(0.75, 1.0))
    );
    assert_eq!(
        AvatarImage::new(texture, vec2(100.0, 200.0)).cover_uv(),
        Rect::from_min_max(pos2(0.0, 0.25), pos2(1.0, 0.75))
    );
    let full = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    assert_eq!(AvatarImage::new(texture, vec2(64.0, 64.0)).cover_uv(), full);
    assert_eq!(AvatarImage::new(texture, vec2(0.0, 0.0)).cover_uv(), full);
}

#[test]
fn image_avatar_paints_a_textured_rect_and_no_letter() {
    let texture = TextureId::Managed(7);
    let (rect, shapes) = AvatarPaint::image_shapes(texture, false);
    assert!(AvatarPaint::letters(&shapes).is_empty());
    let textured: Vec<_> = PaintCapture::rects(&shapes).into_iter().filter(|r| r.brush.is_some()).collect();
    assert_eq!(textured.len(), 1);
    assert_eq!(textured[0].rect, rect);
    assert_eq!(textured[0].brush.as_ref().map(|brush| brush.fill_texture_id), Some(texture));
}

#[test]
fn off_image_is_tinted_darker() {
    let texture = TextureId::Managed(7);
    let tint = |off| {
        let (_, shapes) = AvatarPaint::image_shapes(texture, off);
        PaintCapture::rects(&shapes).into_iter().find(|r| r.brush.is_some()).map(|r| r.fill)
    };
    assert_eq!(tint(false), Some(Color32::WHITE));
    let dim = tint(true).unwrap_or(Color32::WHITE);
    assert!(dim.a() < Color32::WHITE.a() || dim.r() < 255, "off tint {dim:?} is not darker than white");
}
