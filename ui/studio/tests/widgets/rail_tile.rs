use amml_i18n::text::{Text, UserText};
use amml_studio_ui::message::StudioUiMessage;
use amml_studio_ui::widgets::{RailIcon, RailTile, RailTone};
use amml_ui::icon::Icon;
use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Color32, Rect, Shape, pos2, vec2};

use super::headless::Headless;
use crate::support::{PaintCapture, UiProbe};

// the tile's rect in the paint tests
fn tile() -> Rect {
    Rect::from_min_size(pos2(20.0, 20.0), vec2(RailTile::SIDE, RailTile::SIDE))
}

// a label the caller got from the model
fn label(words: &str) -> Text {
    Text::user(UserText::from_model(words.to_owned()))
}

// what a Pause tile paints in `state`
fn paint(state: InteractionState) -> Vec<Shape> {
    PaintCapture::shapes(|painter| {
        RailTile::paint(painter, tile(), RailIcon::Glyph(Icon::Pause), &label("Pause"), RailTone::Normal, state)
    })
}

// the fill of the tile's own rect (not its blurred shadow or its border)
fn fill(shapes: &[Shape]) -> Color32 {
    shapes
        .iter()
        .find_map(|shape| match shape {
            Shape::Rect(rect) if rect.rect == tile() && rect.blur_width == 0.0 && rect.fill.a() > 0 => Some(rect.fill),
            _ => None,
        })
        .unwrap_or(Color32::TRANSPARENT)
}

#[test]
fn rail_tile_side_is_the_large_tile_token() {
    let mut headless = Headless::new();

    let rect =
        headless.rect(|ui| RailTile::show(ui, RailIcon::Glyph(Icon::Pause), &label("Pause"), RailTone::Normal, true));

    assert_eq!(RailTile::SIDE, <spacing::TileLarge as SpacingToken>::VALUE);
    assert_eq!(rect.size(), egui::vec2(RailTile::SIDE, RailTile::SIDE));
}

#[test]
fn rail_tile_states_follow_the_state_model() {
    // Paper `A89-0`: rest is `--color-card`, hover `--color-bg-hover`, pressed `--color-bg-active`
    assert_eq!(fill(&paint(InteractionState::Rest)), <color::Card as ColorToken>::VALUE);
    assert_eq!(fill(&paint(InteractionState::Hover)), <color::BgHover as ColorToken>::VALUE);
    assert_eq!(fill(&paint(InteractionState::Pressed)), <color::BgActive as ColorToken>::VALUE);
    // focus adds the ring; disabled fades the tile to `--opacity-muted`
    let ring = <color::FocusRing as ColorToken>::VALUE;
    let focused = paint(InteractionState::Focused);
    assert!(focused.iter().any(|shape| matches!(shape, Shape::Rect(rect) if rect.fill == ring)), "no focus ring");
    let disabled = fill(&paint(InteractionState::Disabled));
    assert!(disabled.a() < <color::Card as ColorToken>::VALUE.a(), "{disabled:?}");
}

#[test]
fn enabled_rail_tile_reports_click() {
    let mut headless = Headless::new();
    let record = label("Record");
    let rect = headless.rect(|ui| RailTile::show(ui, RailIcon::RecordDot, &record, RailTone::Normal, true));

    let clicked = headless.click(Headless::centre(rect), |ui| {
        RailTile::show(ui, RailIcon::RecordDot, &record, RailTone::Normal, true).clicked()
    });

    assert!(clicked);
}

#[test]
fn disabled_rail_tile_ignores_click() {
    let mut headless = Headless::new();
    let pause = label("Pause");
    let rect = headless.rect(|ui| RailTile::show(ui, RailIcon::Glyph(Icon::Pause), &pause, RailTone::Normal, false));

    let clicked = headless.click(Headless::centre(rect), |ui| {
        RailTile::show(ui, RailIcon::Glyph(Icon::Pause), &pause, RailTone::Normal, false).clicked()
    });

    assert!(!clicked);
}

// the label is `Text`: a message is resolved in the locale of the context, not shown as its English msgid
#[test]
fn rail_tile_label_follows_the_context_locale() {
    let mut probe = UiProbe::pseudo(400.0, 300.0);
    let pause = Text::message(StudioUiMessage::Pause);

    let shapes = probe.shapes(|ui| {
        RailTile::show(ui, RailIcon::Glyph(Icon::Pause), &pause, RailTone::Normal, true);
    });

    let words = UiProbe::words(&shapes);
    assert!(words.iter().any(|word| word.starts_with('[') && word != "Pause"), "{words:?}");
    assert!(!words.iter().any(|word| word == "Pause"), "{words:?}");
}
