use amml_i18n::text::{Text, UserText};
use amml_ui::font::FontSource;
use amml_ui::state::DisabledReason;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{KeyChord, Tooltip};
use egui::{Rect, Sense, Shape, pos2};
use egui_kittest::Harness;

use crate::support::{PaintCapture, Sample, ShapeText};

#[test]
fn shortcut_shows_a_small_key_cap() {
    let tooltip = Tooltip::new(Sample::text("Play")).shortcut(KeyChord::new(vec![Sample::text("Space")]));
    let shapes = PaintCapture::shapes(|painter| {
        tooltip.paint(painter, pos2(20.0, 20.0));
    });
    assert!(ShapeText::contains(&shapes, "Play") && ShapeText::contains(&shapes, "Space"));
    let cap = PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == color::BgField::VALUE);
    assert_eq!(cap.map(|r| r.rect.height()), Some(20.0));
}

#[test]
fn tooltip_is_a_floating_raised_bubble() {
    let text = Text::user(UserText::from_model("Loop playback".to_owned()));
    let shapes = PaintCapture::shapes(|painter| {
        Tooltip::new(text.clone()).paint(painter, pos2(20.0, 20.0));
    });
    let rects: Vec<_> =
        shapes.iter().filter_map(|s| if let Shape::Rect(r) = s { Some(r.clone()) } else { None }).collect();
    let bubble = rects.iter().find(|r| r.fill == <color::BgRaised as ColorToken>::VALUE);
    assert!(bubble.is_some(), "no raised fill: {rects:?}");
    let has_shadow = rects.iter().any(|r| r.fill == <color::Shadow as ColorToken>::VALUE);
    assert!(has_shadow, "no floating shadow");
    let strong = rects.iter().any(|r| r.stroke.color == <color::LineStrong as ColorToken>::VALUE);
    assert!(strong, "no strong border");
    assert!(ShapeText::contains(&shapes, "Loop playback"));
}

#[test]
fn disabled_control_shows_its_reason() {
    let target = Rect::from_min_max(pos2(20.0, 20.0), pos2(120.0, 56.0));
    let reason = DisabledReason::new(Text::user(UserText::from_model("Select a key first".to_owned())));
    let mut harness = Harness::new_ui(move |ui| {
        let response = ui.allocate_rect(target, Sense::hover());
        reason.tooltip().attach(response);
    });
    Theme::install(&harness.ctx, FontSource::BuiltIn);
    harness.ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
    harness.hover_at(target.center());
    harness.run();
    let shapes: Vec<Shape> = harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect();
    assert!(ShapeText::contains(&shapes, "Select a key first"), "{:?}", ShapeText::all(&shapes));
}
