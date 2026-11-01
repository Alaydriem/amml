use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{Dropdown, DropdownOption};
use egui::{Key, Rect, Shape, pos2};

use crate::support::{ClickProbe, PaintCapture, Sample, ShapeText};

struct Weather;

impl Weather {
    fn options() -> Vec<DropdownOption> {
        let mut options: Vec<DropdownOption> = ["Recorded", "Clear", "Rain", "Thunder"]
            .iter()
            .map(|word| DropdownOption::new(Sample::text(word)))
            .collect();
        options.push(DropdownOption::new(Sample::text("Snow")).enabled(false));
        options
    }
}

#[test]
fn click_opens_and_pick_marks_changed() {
    let mut harness = ClickProbe::harness_with(Some(2), |ui, selected| {
        Dropdown::new(egui::Id::unique(1), selected, Weather::options()).show(ui, 220.0)
    });
    ClickProbe::click(&mut harness);
    let Some(thunder) = ClickProbe::text_rect(&harness, "Thunder") else { panic!("the list did not open") };
    ClickProbe::click_at(&mut harness, thunder.center());
    assert_eq!(harness.state().value, Some(3));
    assert_eq!(harness.state().changes, 1);
    assert!(!ClickProbe::texts(&harness).contains(&"Recorded".to_owned()), "the list did not close");
}

#[test]
fn disabled_option_cannot_be_picked() {
    let mut harness = ClickProbe::harness_with(Some(2), |ui, selected| {
        Dropdown::new(egui::Id::unique(1), selected, Weather::options()).show(ui, 220.0)
    });
    ClickProbe::click(&mut harness);
    let Some(snow) = ClickProbe::text_rect(&harness, "Snow") else { panic!("the list did not open") };
    ClickProbe::click_at(&mut harness, snow.center());
    assert_eq!(harness.state().value, Some(2));
    assert_eq!(harness.state().changes, 0);
}

#[test]
fn escape_closes_without_change() {
    let mut harness = ClickProbe::harness_with(Some(2), |ui, selected| {
        Dropdown::new(egui::Id::unique(1), selected, Weather::options()).show(ui, 220.0)
    });
    ClickProbe::click(&mut harness);
    assert!(ClickProbe::texts(&harness).contains(&"Thunder".to_owned()));
    harness.key_press(Key::Escape);
    harness.run();
    assert!(!ClickProbe::texts(&harness).contains(&"Thunder".to_owned()));
    assert_eq!(harness.state().value, Some(2));
}

#[test]
fn keys_move_and_pick() {
    let mut harness = ClickProbe::harness_with(Some(0), |ui, selected| {
        Dropdown::new(egui::Id::unique(1), selected, Weather::options()).show(ui, 220.0)
    });
    ClickProbe::click(&mut harness);
    for key in [Key::ArrowDown, Key::ArrowDown, Key::Enter] {
        harness.key_press(key);
        harness.run();
    }
    assert_eq!(harness.state().value, Some(2));
}

#[test]
fn placeholder_when_none() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(200.0, 54.0));
    let mut selected = None;
    let shapes = PaintCapture::shapes(|painter| {
        Dropdown::new(egui::Id::NULL, &mut selected, Weather::options())
            .placeholder(Sample::text("Choose a scene"))
            .paint(painter, rect, InteractionState::Rest, false);
    });
    let text = shapes.iter().find_map(|s| match s {
        Shape::Text(t) if t.galley.text() == "Choose a scene" => Some(t.galley.job.sections[0].format.color),
        _ => None,
    });
    assert_eq!(text, Some(color::TextFaint::VALUE));
}

#[test]
fn trigger_keeps_its_border_and_open_is_active() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(200.0, 54.0));
    let mut selected = Some(2);
    let shapes = PaintCapture::shapes(|painter| {
        Dropdown::new(egui::Id::NULL, &mut selected, Weather::options()).paint(
            painter,
            rect,
            InteractionState::Rest,
            true,
        );
    });
    let rects = PaintCapture::rects(&shapes);
    assert!(rects.iter().any(|r| r.rect == rect && r.fill == color::BgActive::VALUE), "open is not active");
    assert!(rects.iter().any(|r| r.rect == rect && r.stroke.color == color::LineStrong::VALUE));
    assert!(ShapeText::contains(&shapes, "\u{E70E}"), "open shows the up chevron");
}

#[test]
fn two_dropdowns_open_independently() {
    let mut harness = ClickProbe::harness_with((Some(0), Some(1)), |ui, (a, b)| {
        let first = Dropdown::new(egui::Id::unique(1), a, Weather::options()).show(ui, 220.0);
        Dropdown::new(egui::Id::unique(2), b, Weather::options()).show(ui, 220.0);
        first
    });
    ClickProbe::click(&mut harness);
    let texts = ClickProbe::texts(&harness);
    assert_eq!(texts.iter().filter(|t| *t == "Thunder").count(), 1, "{texts:?}");
}

#[test]
fn keys_in_an_open_list_do_not_reach_the_next_widget() {
    // the value is the pick and the clicks of a button under the dropdown
    let mut harness = ClickProbe::harness_with((Some(0), 0_usize), |ui, (selected, clicks)| {
        let dropdown = Dropdown::new(egui::Id::unique(1), selected, Weather::options()).show(ui, 220.0);
        *clicks += usize::from(ui.button("Delete").clicked());
        dropdown
    });
    ClickProbe::tab_and_press(&mut harness, Key::Space);
    for key in [Key::ArrowDown, Key::Enter] {
        harness.key_press(key);
        harness.run();
    }
    assert_eq!(harness.state().value, (Some(1), 0));
}
