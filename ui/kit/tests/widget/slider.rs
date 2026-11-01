use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::Slider;
use egui::{Color32, Key, Rect, Shape, pos2};

use crate::support::{ClickProbe, PaintCapture};

struct SliderPaint;

impl SliderPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(260.0, 40.0));

    fn shapes(value: f64, ticks: bool, state: InteractionState) -> Vec<Shape> {
        let mut value = value;
        PaintCapture::shapes(|painter| {
            Slider::new(&mut value, 0.0..=4.0).step(1.0).ticks(ticks).paint(painter, Self::RECT, state);
        })
    }

    fn circles(shapes: &[Shape]) -> Vec<(f32, Color32)> {
        shapes.iter().filter_map(|s| if let Shape::Circle(c) = s { Some((c.radius, c.fill)) } else { None }).collect()
    }
}

#[test]
fn snaps_to_step() {
    let mut harness =
        ClickProbe::harness_with(0.0_f64, |ui, value| Slider::new(value, 0.0..=4.0).step(1.0).show(ui, 240.0));
    // 62 % of the track
    let rect = harness.state().rect;
    let x = rect.left() + 7.0 + (rect.width() - 14.0) * 0.62;
    let at = pos2(x, rect.center().y);
    harness.hover_at(at);
    harness.drag_at(at);
    harness.run();
    harness.drop_at(at);
    harness.run();
    assert_eq!(harness.state().value, 2.0);
    assert!(harness.state().changes >= 1);
}

#[test]
fn empty_range_does_not_panic_or_change() {
    let mut harness = ClickProbe::harness_with(2.0_f64, |ui, value| Slider::new(value, 2.0..=2.0).show(ui, 240.0));
    ClickProbe::click(&mut harness);
    ClickProbe::tab_and_press(&mut harness, Key::ArrowRight);
    assert_eq!(harness.state().value, 2.0);
    assert_eq!(harness.state().changes, 0);
}

#[test]
fn nan_value_is_clamped_to_start() {
    let shapes = SliderPaint::shapes(f64::NAN, false, InteractionState::Rest);
    // the knob sits at the start of the track: 7 px in from the left
    let knob = shapes.iter().find_map(|s| if let Shape::Circle(c) = s { Some(c.center) } else { None });
    assert_eq!(knob.map(|c| c.x), Some(SliderPaint::RECT.left() + 7.0));
}

#[test]
fn arrow_key_moves_one_step() {
    let mut harness =
        ClickProbe::harness_with(1.0_f64, |ui, value| Slider::new(value, 0.0..=4.0).step(1.0).show(ui, 240.0));
    ClickProbe::tab_and_press(&mut harness, Key::ArrowRight);
    assert_eq!(harness.state().value, 2.0);
    harness.key_press(Key::ArrowLeft);
    harness.run();
    harness.key_press(Key::ArrowLeft);
    harness.run();
    assert_eq!(harness.state().value, 0.0);
}

#[test]
fn knob_grows_on_hover_and_halo_only_while_dragging() {
    let knob = |state| {
        let circles = SliderPaint::circles(&SliderPaint::shapes(2.0, false, state));
        circles.iter().find(|(_, fill)| *fill == color::OnAccent::VALUE).map(|(r, _)| *r)
    };
    assert_eq!(knob(InteractionState::Rest), Some(7.0));
    assert_eq!(knob(InteractionState::Hover), Some(8.0));
    let halo = |state| {
        SliderPaint::circles(&SliderPaint::shapes(2.0, false, state))
            .iter()
            .any(|(r, fill)| *r == 14.0 && *fill == color::AccentSoft::VALUE)
    };
    assert!(halo(InteractionState::Pressed));
    assert!(!halo(InteractionState::Hover));
}

#[test]
fn ticks_left_of_the_knob_are_accent() {
    let shapes = SliderPaint::shapes(3.0, true, InteractionState::Rest);
    let ticks: Vec<Color32> =
        SliderPaint::circles(&shapes).into_iter().filter(|(r, _)| *r == 1.5).map(|(_, fill)| fill).collect();
    let accent = color::Accent::VALUE;
    let strong = color::LineStrong::VALUE;
    assert_eq!(ticks, vec![accent, accent, accent, accent, strong]);
}

#[test]
fn track_and_fill_follow_the_state() {
    let fills = |state| {
        PaintCapture::rects(&SliderPaint::shapes(2.0, false, state)).into_iter().map(|r| r.fill).collect::<Vec<_>>()
    };
    let rest = fills(InteractionState::Rest);
    assert!(rest.contains(&color::BgHover::VALUE) && rest.contains(&color::Accent::VALUE));
    let hover = fills(InteractionState::Hover);
    assert!(hover.contains(&color::BgActive::VALUE) && hover.contains(&color::AccentHover::VALUE));
}

#[test]
fn arrow_keys_stay_on_the_focused_slider() {
    // egui moves focus on a bare arrow key unless the focused widget keeps it
    let mut harness = ClickProbe::harness_with((2.0_f64, 2.0_f64), |ui, (a, b)| {
        let first = Slider::new(a, 0.0..=4.0).step(1.0).show(ui, 240.0);
        Slider::new(b, 0.0..=4.0).step(1.0).show(ui, 240.0);
        first
    });
    ClickProbe::tab_and_press(&mut harness, Key::ArrowDown);
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert_eq!(harness.state().value, (0.0, 2.0));
}
