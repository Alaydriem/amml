use amml_studio_ui::component::easing::{
    EasingFamily, EasingPopover, EasingPopoverAction, EasingPopoverView, EasingPreset, HandleIndex,
};
use amml_studio_ui::tokens::selector::TrackColor;
use egui::{Rect, Shape, pos2, vec2};

use crate::glyph::words;
use crate::support::UiProbe;

fn preset(name: &str, family: EasingFamily) -> EasingPreset {
    EasingPreset { name: words(name), samples: (0..=16).map(|step| step as f32 / 16.0).collect(), family }
}

fn view(handles: Option<[egui::Pos2; 2]>) -> EasingPopoverView {
    EasingPopoverView {
        subtitle: words("Camera key · applies to 1 selected key"),
        track: TrackColor::Camera,
        samples: (0..=16).map(|step| step as f32 / 16.0).collect(),
        handles,
        family: EasingFamily::All,
        presets: vec![
            preset("Smooth", EasingFamily::InOut),
            preset("In Quad", EasingFamily::In),
            preset("Out Quad", EasingFamily::Out),
        ],
        selected: None,
        saved: vec![preset("Soft landing", EasingFamily::Other)],
        selected_saved: None,
    }
}

// the popover's card rect, from one quiet frame
fn card(probe: &mut UiProbe, view: &EasingPopoverView) -> Rect {
    let (_, _) = probe.frame(Vec::new(), |ui| EasingPopover::new(view).show(ui));
    let ((_, rect), _) = probe.frame(Vec::new(), |ui| (EasingPopover::new(view).show(ui), ui.min_rect()));
    rect
}

fn moves(actions: &[EasingPopoverAction]) -> Vec<(HandleIndex, egui::Pos2)> {
    actions
        .iter()
        .filter_map(|action| match action {
            EasingPopoverAction::MoveHandle(handle, at) => Some((*handle, *at)),
            _ => None,
        })
        .collect()
}

#[test]
fn long_lists_scroll_to_their_last_preset() {
    let mut view = view(None);
    view.saved = (0..10).map(|index| preset(&format!("Saved {index}"), EasingFamily::Other)).collect();
    view.presets = (0..30).map(|index| preset(&format!("Preset {index}"), EasingFamily::InOut)).collect();
    let mut probe = UiProbe::sized(900.0, 640.0);
    let card = card(&mut probe, &view);
    let show = |ui: &mut egui::Ui| EasingPopover::new(&view).show(ui);
    for (first, last, picked) in [
        ("Saved 0", "Saved 9", EasingPopoverAction::PickSaved(9)),
        ("Preset 0", "Preset 29", EasingPopoverAction::PickPreset(29)),
    ] {
        let shapes = probe.shapes(show);
        let first = UiProbe::text_rect(&shapes, first).unwrap_or_else(|| panic!("no {first}"));
        probe.wheel(first.center(), vec2(0.0, -2000.0), show);
        let shapes = probe.shapes(show);
        let last_rect = UiProbe::text_rect(&shapes, last)
            .filter(|rect| card.contains_rect(*rect))
            .unwrap_or_else(|| panic!("{last} is not in the card after a scroll: {:?}", UiProbe::texts(&shapes)));
        let actions = probe.click(last_rect.center(), show);
        assert!(actions.contains(&picked), "{last}: {actions:?}");
    }
}

#[test]
fn handle_drag_sends_move_handle() {
    let view = view(Some([pos2(0.4, 0.2), pos2(0.6, 0.9)]));
    let mut probe = UiProbe::sized(900.0, 640.0);
    let card = card(&mut probe, &view);
    let from = EasingPopover::new(&view).handle_place(card, HandleIndex::First).expect("a custom curve has handles");
    let actions = probe.drag(from, from + vec2(10.0, 0.0), |ui| EasingPopover::new(&view).show(ui)).concat();
    let moved = moves(&actions);
    let (handle, at) = moved.last().copied().expect("the drag moves the handle");
    assert_eq!(handle, HandleIndex::First);
    assert!(at.x > 0.4, "{at:?}");
}

#[test]
fn overshooting_handle_is_grabbed_at_its_painted_knob() {
    let view = view(Some([pos2(0.4, 0.2), pos2(0.6, 2.0)]));
    let mut probe = UiProbe::sized(900.0, 640.0);
    card(&mut probe, &view);
    let shapes = probe.shapes(|ui| EasingPopover::new(&view).show(ui));
    // the knobs are the two radius 4 rings; the overshooting one is the higher
    let knob = shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Circle(circle) if circle.radius == 4.0 => Some(circle.center),
            _ => None,
        })
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("two knobs");
    let actions = probe.drag(knob, knob + vec2(0.0, 20.0), |ui| EasingPopover::new(&view).show(ui)).concat();
    let moved = moves(&actions);
    assert!(moved.iter().any(|(handle, _)| *handle == HandleIndex::Second), "{actions:?}");
}

#[test]
fn handles_stay_in_the_unit_box_on_x() {
    let view = view(Some([pos2(0.4, 0.2), pos2(0.6, 0.9)]));
    let mut probe = UiProbe::sized(900.0, 640.0);
    let card = card(&mut probe, &view);
    let from = EasingPopover::new(&view).handle_place(card, HandleIndex::First).expect("a custom curve has handles");
    let actions = probe.drag(from, from - vec2(400.0, 0.0), |ui| EasingPopover::new(&view).show(ui)).concat();
    let (_, at) = moves(&actions).last().copied().expect("the drag moves the handle");
    assert_eq!(at.x, 0.0);
}

#[test]
fn preset_click_picks_it() {
    let view = view(None);
    let mut probe = UiProbe::sized(900.0, 640.0);
    let shapes = probe.shapes(|ui| EasingPopover::new(&view).show(ui));
    let card = UiProbe::text_rect(&shapes, "In Quad").expect("the In Quad card is painted");
    let actions = probe.click(card.center(), |ui| EasingPopover::new(&view).show(ui));
    assert_eq!(actions, vec![EasingPopoverAction::PickPreset(1)]);
}

#[test]
fn named_curve_for_three_keys_has_no_handles() {
    let view = view(None);
    let mut probe = UiProbe::sized(900.0, 640.0);
    let card = card(&mut probe, &view);
    assert!(EasingPopover::new(&view).handle_place(card, HandleIndex::First).is_none());
    let shapes = probe.shapes(|ui| EasingPopover::new(&view).show(ui));
    let hue = TrackColor::Camera.color();
    let knobs =
        shapes.iter().filter(|shape| matches!(shape, Shape::Circle(circle) if circle.stroke.color == hue)).count();
    assert_eq!(knobs, 0);
}
