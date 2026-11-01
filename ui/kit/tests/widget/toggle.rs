use amml_ui::state::{DisabledReason, InteractionState};
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::Toggle;
use egui::{Color32, Key, Pos2, Rect, Shape, pos2, vec2};

use crate::support::{ClickProbe, PaintCapture, Sample};

struct TogglePaint;

impl TogglePaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(54.0, 40.0));

    // (track fill, knob fill, knob centre)
    fn look(on: bool, state: InteractionState) -> (Color32, Color32, Pos2) {
        let mut value = on;
        let shapes = PaintCapture::shapes(|painter| Toggle::new(&mut value).paint(painter, Self::RECT, state));
        let track =
            PaintCapture::rects(&shapes).iter().find(|r| r.rect == Self::RECT).map_or(Color32::TRANSPARENT, |r| r.fill);
        let knob = shapes.iter().find_map(|shape| match shape {
            Shape::Circle(circle) => Some((circle.fill, circle.center)),
            _ => None,
        });
        let (knob_fill, centre) = knob.unwrap_or((Color32::TRANSPARENT, Pos2::ZERO));
        (track, knob_fill, centre)
    }
}

#[test]
fn off_and_on_ladders() {
    let off = [
        (InteractionState::Rest, color::BgHover::VALUE, color::TextDim::VALUE),
        (InteractionState::Hover, color::BgActive::VALUE, color::Text::VALUE),
        (InteractionState::Pressed, color::LineStrong::VALUE, color::Text::VALUE),
        (InteractionState::Focused, color::BgHover::VALUE, color::TextDim::VALUE),
    ];
    for (state, track, knob) in off {
        let (t, k, _) = TogglePaint::look(false, state);
        assert_eq!((t, k), (track, knob), "off {state:?}");
    }
    let on = [
        (InteractionState::Rest, color::Accent::VALUE),
        (InteractionState::Hover, color::AccentHover::VALUE),
        (InteractionState::Pressed, color::AccentPress::VALUE),
        (InteractionState::Focused, color::Accent::VALUE),
    ];
    for (state, track) in on {
        let (t, k, _) = TogglePaint::look(true, state);
        assert_eq!((t, k), (track, color::OnAccent::VALUE), "on {state:?}");
    }
}

#[test]
fn knob_is_at_the_end_when_settled() {
    let (_, _, off) = TogglePaint::look(false, InteractionState::Rest);
    let (_, _, on) = TogglePaint::look(true, InteractionState::Rest);
    // knob 14 inset 3 in a 34 x 20 track: centres at 10 and 24 from the left
    assert_eq!(off, pos2(30.0, 30.0));
    assert_eq!(on, pos2(44.0, 30.0));
}

#[test]
fn click_flips_and_marks_changed() {
    let mut harness = ClickProbe::harness_with(false, |ui, on| Toggle::new(on).show(ui));
    ClickProbe::click(&mut harness);
    assert_eq!(harness.state().clicks, 1);
    assert_eq!(harness.state().changes, 1);
    assert!(harness.state().value);
    assert_eq!(harness.state().rect.size(), vec2(34.0, 20.0));
}

#[test]
fn disabled_toggle_does_not_flip() {
    let mut harness = ClickProbe::harness_with(false, |ui, on| {
        Toggle::new(on).enabled(false, DisabledReason::new(Sample::text("Turn on the mod first"))).show(ui)
    });
    ClickProbe::click(&mut harness);
    ClickProbe::tab_and_press(&mut harness, Key::Space);
    assert!(!harness.state().value);
    assert_eq!(harness.state().changes, 0);
}

#[test]
fn space_flips_a_focused_toggle() {
    let mut harness = ClickProbe::harness_with(false, |ui, on| Toggle::new(on).show(ui));
    ClickProbe::tab_and_press(&mut harness, Key::Space);
    assert!(harness.state().value);
}

#[test]
fn focus_ring_is_round() {
    let mut value = true;
    let shapes = PaintCapture::shapes(|painter| {
        Toggle::new(&mut value).paint(painter, TogglePaint::RECT, InteractionState::Focused)
    });
    let ring = PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == color::FocusRing::VALUE);
    assert_eq!(ring.map(|r| r.rect), Some(TogglePaint::RECT.expand(4.0)));
}

#[test]
fn paint_amount_puts_the_knob_part_way() {
    let shapes =
        PaintCapture::shapes(|painter| Toggle::paint_amount(painter, TogglePaint::RECT, InteractionState::Rest, 0.5));
    let centre = shapes.iter().find_map(|shape| match shape {
        Shape::Circle(circle) => Some(circle.center),
        _ => None,
    });
    // halfway between the settled centres 30 and 44
    assert_eq!(centre, Some(pos2(37.0, 30.0)));
}
