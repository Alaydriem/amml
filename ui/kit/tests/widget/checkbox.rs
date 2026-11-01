use amml_ui::state::{DisabledReason, InteractionState, Selection};
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::Checkbox;
use egui::{Color32, Key, Rect, pos2};

use crate::support::{ClickProbe, PaintCapture, Sample};

struct BoxPaint;

impl BoxPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(36.0, 36.0));

    // (fill, border colour) of the box
    fn look(value: Selection, state: InteractionState) -> (Color32, Option<Color32>) {
        let mut value = value;
        let shapes = PaintCapture::shapes(|painter| Checkbox::new(&mut value).paint(painter, Self::RECT, state));
        let rects = PaintCapture::rects(&shapes);
        let fill = rects
            .iter()
            .find(|r| r.rect == Self::RECT && r.fill != Color32::TRANSPARENT)
            .map_or(Color32::TRANSPARENT, |r| r.fill);
        let border = rects.iter().find(|r| r.rect == Self::RECT && r.stroke.width > 0.0).map(|r| r.stroke.color);
        (fill, border)
    }
}

#[test]
fn unchecked_has_a_border_checked_does_not() {
    assert_eq!(
        BoxPaint::look(Selection::Off, InteractionState::Rest),
        (color::BgField::VALUE, Some(color::LineStrong::VALUE))
    );
    assert_eq!(
        BoxPaint::look(Selection::Off, InteractionState::Hover),
        (color::BgHover::VALUE, Some(color::LineStrong::VALUE))
    );
    assert_eq!(BoxPaint::look(Selection::On, InteractionState::Rest), (color::Accent::VALUE, None));
    assert_eq!(BoxPaint::look(Selection::Mixed, InteractionState::Pressed), (color::AccentPress::VALUE, None));
}

#[test]
fn mixed_click_becomes_on() {
    let mut harness = ClickProbe::harness_with(Selection::Mixed, |ui, value| Checkbox::new(value).show(ui));
    ClickProbe::click(&mut harness);
    assert_eq!(harness.state().value, Selection::On);
    ClickProbe::click(&mut harness);
    assert_eq!(harness.state().value, Selection::Off);
}

#[test]
fn row_click_toggles() {
    let mut harness = ClickProbe::harness_with(Selection::Off, |ui, value| {
        Checkbox::new(value).label(Sample::text("Camera")).show(ui)
    });
    let rect = harness.state().rect;
    assert_eq!(rect.height(), 34.0);
    let on_label = rect.left_center() + egui::vec2(120.0, 0.0);
    harness.hover_at(on_label);
    harness.drag_at(on_label);
    harness.run();
    harness.drop_at(on_label);
    harness.run();
    assert_eq!(harness.state().value, Selection::On);
    assert_eq!(harness.state().changes, 1);
}

#[test]
fn disabled_row_is_muted_and_does_not_toggle() {
    let mut harness = ClickProbe::harness_with(Selection::Off, |ui, value| {
        Checkbox::new(value)
            .label(Sample::text("Timelapse"))
            .enabled(false, DisabledReason::new(Sample::text("No timelapse in this replay")))
            .show(ui)
    });
    ClickProbe::click(&mut harness);
    ClickProbe::tab_and_press(&mut harness, Key::Space);
    assert_eq!(harness.state().value, Selection::Off);
    let (fill, _) = BoxPaint::look(Selection::Off, InteractionState::Disabled);
    assert!(fill.a() < 255);
}

#[test]
fn check_and_dash_are_on_accent() {
    let mut on = Selection::On;
    let shapes =
        PaintCapture::shapes(|painter| Checkbox::new(&mut on).paint(painter, BoxPaint::RECT, InteractionState::Rest));
    let check = PaintCapture::texts(&shapes);
    assert_eq!(check.first().map(|(text, size, _)| (text.clone(), *size)), Some(("\u{E73E}".to_owned(), 12.0)));
    let mut mixed = Selection::Mixed;
    let shapes = PaintCapture::shapes(|painter| {
        Checkbox::new(&mut mixed).paint(painter, BoxPaint::RECT, InteractionState::Rest)
    });
    let dash = PaintCapture::rects(&shapes).into_iter().find(|r| r.rect.height() == 2.0);
    assert_eq!(dash.map(|r| (r.rect.width(), r.fill)), Some((8.0, color::OnAccent::VALUE)));
}
