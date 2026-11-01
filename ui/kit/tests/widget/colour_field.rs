use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{ColourField, ColourValue, FieldLook};
use egui::{Event, Key, Rect, Shape, pos2};

use crate::support::{ClickProbe, PaintCapture, ShapeText};

#[test]
fn parse_accepts_six_and_eight_digits_only() {
    assert_eq!(ColourValue::parse("#4fd1a5"), Some(ColourValue { rgba: [0x4F, 0xD1, 0xA5, 0xFF], alpha: false }));
    assert_eq!(ColourValue::parse("#FFFFFF40"), Some(ColourValue { rgba: [0xFF, 0xFF, 0xFF, 0x40], alpha: true }));
    for bad in ["#FFG", "FFFFFF", "#12345", "#1234567", "", "#GGGGGG"] {
        assert_eq!(ColourValue::parse(bad), None, "{bad}");
    }
}

#[test]
fn hex_is_upper_case() {
    assert_eq!(ColourValue { rgba: [0x4F, 0xD1, 0xA5, 0xFF], alpha: false }.hex(), "#4FD1A5");
    assert_eq!(ColourValue { rgba: [0xFF, 0xFF, 0xFF, 0x40], alpha: true }.hex(), "#FFFFFF40");
}

#[test]
fn alpha_swatch_is_split() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(200.0, 56.0));
    let mut value = ColourValue { rgba: [0xFF, 0xFF, 0xFF, 0x40], alpha: true };
    let shapes = PaintCapture::shapes(|painter| {
        ColourField::new(egui::Id::NULL, &mut value).paint(
            painter,
            rect,
            InteractionState::Rest,
            &FieldLook::default(),
        );
    });
    let rects = PaintCapture::rects(&shapes);
    assert!(rects.iter().any(|r| r.fill == color::CheckerLight::VALUE));
    assert!(rects.iter().any(|r| r.fill == color::CheckerDark::VALUE));
    assert!(rects.iter().any(|r| r.fill == egui::Color32::WHITE), "no solid half");
}

#[test]
fn invalid_keeps_last_colour_and_shows_message() {
    let mut harness =
        ClickProbe::harness_with(ColourValue { rgba: [0x4F, 0xD1, 0xA5, 0xFF], alpha: false }, |ui, value| {
            ColourField::new(egui::Id::unique(1), value).show(ui)
        });
    ClickProbe::click(&mut harness);
    harness.event(Event::Text("#FFG".into()));
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().value.rgba, [0x4F, 0xD1, 0xA5, 0xFF]);
    let shapes: Vec<Shape> = harness.output().shapes.iter().map(|c| c.shape.clone()).collect();
    assert!(ShapeText::contains(&shapes, "Use #RRGGBB or #RRGGBBAA."), "{:?}", ShapeText::all(&shapes));
}

#[test]
fn valid_hex_updates_the_value() {
    let mut harness = ClickProbe::harness_with(ColourValue { rgba: [0, 0, 0, 0xFF], alpha: false }, |ui, value| {
        ColourField::new(egui::Id::unique(1), value).show(ui)
    });
    ClickProbe::click(&mut harness);
    harness.event(Event::Text("#4FD1A5".into()));
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().value.rgba, [0x4F, 0xD1, 0xA5, 0xFF]);
}

#[test]
fn hover_shows_field_and_border() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(200.0, 56.0));
    let mut value = ColourValue { rgba: [0x4F, 0xD1, 0xA5, 0xFF], alpha: false };
    let shapes = PaintCapture::shapes(|painter| {
        ColourField::new(egui::Id::NULL, &mut value).paint(
            painter,
            rect,
            InteractionState::Hover,
            &FieldLook::default(),
        );
    });
    let rects = PaintCapture::rects(&shapes);
    assert!(rects.iter().any(|r| r.rect == rect && r.fill == color::BgField::VALUE));
    assert!(rects.iter().any(|r| r.rect == rect && r.stroke.color == color::LineStrong::VALUE));
}
