use amml_ui::state::InteractionState;
use amml_ui::tokens::{OpacityToken, opacity};
use egui::{Color32, Key, Rect, Shape, pos2};

use crate::support::{PaintCapture, StateProbe};

#[test]
fn disabled_wins_over_everything() {
    let mut harness = StateProbe::harness(false);
    harness.hover_at(StateProbe::centre());
    harness.drag_at(StateProbe::centre());
    harness.run();
    assert_eq!(*harness.state(), Some(InteractionState::Disabled));
}

#[test]
fn pressed_wins_over_hover() {
    let mut harness = StateProbe::harness(true);
    harness.hover_at(StateProbe::centre());
    harness.drag_at(StateProbe::centre());
    harness.run();
    assert_eq!(*harness.state(), Some(InteractionState::Pressed));
}

#[test]
fn hover_without_press_is_hover() {
    let mut harness = StateProbe::harness(true);
    harness.hover_at(StateProbe::centre());
    harness.run();
    assert_eq!(*harness.state(), Some(InteractionState::Hover));
}

#[test]
fn focus_without_pointer_is_focused() {
    let mut harness = StateProbe::harness(true);
    harness.hover_at(StateProbe::outside());
    harness.run();
    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(*harness.state(), Some(InteractionState::Focused));
}

#[test]
fn untouched_is_rest() {
    let mut harness = StateProbe::harness(true);
    harness.run();
    assert_eq!(*harness.state(), Some(InteractionState::Rest));
}

#[test]
fn disabled_painter_is_muted() {
    let rect = Rect::from_min_max(pos2(10.0, 10.0), pos2(50.0, 50.0));
    let shapes = PaintCapture::shapes(|painter| {
        InteractionState::Disabled.painter(painter).rect_filled(rect, 0, Color32::WHITE);
    });
    let fill = shapes.iter().find_map(|shape| if let Shape::Rect(r) = shape { Some(r.fill) } else { None });
    let muted = (255.0 * <opacity::Muted as OpacityToken>::VALUE).round() as u8;
    assert_eq!(fill.map(|fill| fill.a()), Some(muted));
}

#[test]
fn rest_painter_is_not_muted() {
    let rect = Rect::from_min_max(pos2(10.0, 10.0), pos2(50.0, 50.0));
    let shapes = PaintCapture::shapes(|painter| {
        InteractionState::Rest.painter(painter).rect_filled(rect, 0, Color32::WHITE);
    });
    let fill = shapes.iter().find_map(|shape| if let Shape::Rect(r) = shape { Some(r.fill) } else { None });
    assert_eq!(fill, Some(Color32::WHITE));
}
