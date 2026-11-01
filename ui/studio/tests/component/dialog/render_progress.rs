use amml_studio_ui::component::dialog::{JobStatus, RenderProgressAction, RenderProgressCard, RenderProgressView};
use amml_ui::draw::TypeSize;
use amml_ui::tokens::{ColorToken, color};
use egui::{Key, Shape};

use crate::glyph::words;
use crate::support::UiProbe;

fn view(status: JobStatus, progress: f32) -> RenderProgressView {
    RenderProgressView {
        title: words("Rendering “Arrival”"),
        file: words("nether-trip_arrival_4k60.mp4"),
        status,
        running: status == JobStatus::Rendering,
        progress,
        frames: words("Frame 106 of 372"),
        time_left: words("About 6 s left"),
    }
}

// the width of the accent progress fill
fn bar(progress: f32) -> f32 {
    let view = view(JobStatus::Rendering, progress);
    let shapes = UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderProgressCard::new(&view).show_card(ui));
    let accent = <color::Accent as ColorToken>::VALUE;
    shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Rect(rect) if rect.fill == accent => Some(rect.rect.width()),
            _ => None,
        })
        .sum()
}

#[test]
fn the_card_shows_the_numbers_and_a_cancel_button() {
    let view = view(JobStatus::Rendering, 0.28);
    let shapes = UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderProgressCard::new(&view).show_card(ui));
    let texts = UiProbe::texts(&shapes);
    for shown in ["Rendering “Arrival”", "Frame 106 of 372", "About 6 s left", "Cancel"] {
        assert!(texts.iter().any(|text| text == shown), "{shown} is missing: {texts:?}");
    }

    let at = UiProbe::text_rect(&shapes, "Cancel").expect("the Cancel button").center();
    let mut probe = UiProbe::sized(1000.0, 800.0);
    assert_eq!(probe.click(at, |ui| RenderProgressCard::new(&view).show_card(ui)), Some(RenderProgressAction::Cancel));
}

#[test]
fn progress_is_clamped() {
    let full = bar(1.0);
    assert!(full > 0.0);
    assert!((bar(0.5) - full / 2.0).abs() < 1.0);
    assert_eq!(bar(f32::NAN), 0.0);
    assert_eq!(bar(-0.5), 0.0);
    assert_eq!(bar(1.5), full);
}

#[test]
fn escape_cancels_a_running_render_and_closes_a_finished_one() {
    let running = view(JobStatus::Rendering, 0.5);
    let mut probe = UiProbe::sized(1280.0, 900.0);
    probe.frame(Vec::new(), |ui| RenderProgressCard::new(&running).show(ui.ctx()));
    let (action, _) =
        probe.frame(vec![UiProbe::key(Key::Escape)], |ui| RenderProgressCard::new(&running).show(ui.ctx()));
    assert_eq!(action, Some(RenderProgressAction::Cancel));

    let done = view(JobStatus::Done, 1.0);
    let shapes = UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderProgressCard::new(&done).show_card(ui));
    let texts = UiProbe::texts(&shapes);
    assert!(texts.iter().any(|text| text == "Close"), "{texts:?}");
    assert!(!texts.iter().any(|text| text == "Cancel"), "{texts:?}");
    let mut probe = UiProbe::sized(1280.0, 900.0);
    probe.frame(Vec::new(), |ui| RenderProgressCard::new(&done).show(ui.ctx()));
    let (action, _) = probe.frame(vec![UiProbe::key(Key::Escape)], |ui| RenderProgressCard::new(&done).show(ui.ctx()));
    assert_eq!(action, Some(RenderProgressAction::Close));
}

#[test]
fn a_click_beside_the_card_does_not_cancel() {
    let running = view(JobStatus::Rendering, 0.5);
    let mut probe = UiProbe::sized(1280.0, 900.0);
    let action = probe.click(egui::pos2(10.0, 10.0), |ui| RenderProgressCard::new(&running).show(ui.ctx()));
    assert_eq!(action, None);
}

#[test]
fn open_queue_asks_for_the_queue_while_it_runs_and_once_it_is_over() {
    for status in [JobStatus::Rendering, JobStatus::Done] {
        let view = view(status, 0.5);
        let shapes = UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderProgressCard::new(&view).show_card(ui));
        let at = UiProbe::text_rect(&shapes, "Open queue").expect("the Open queue button").center();
        let mut probe = UiProbe::sized(1000.0, 800.0);
        assert_eq!(
            probe.click(at, |ui| RenderProgressCard::new(&view).show_card(ui)),
            Some(RenderProgressAction::OpenQueue),
            "{status:?}"
        );
    }
}

#[test]
fn a_paused_card_draws_the_pause_line_whole_and_the_cancel_button() {
    let mut view = view(JobStatus::Rendering, 0.28);
    let pause = "Paused: a game screen is open. The render goes on when the game shows the world again.";
    view.time_left = words(pause);
    let shapes = UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderProgressCard::new(&view).show_card(ui));
    let texts = UiProbe::texts(&shapes);
    assert!(texts.iter().any(|text| text == pause), "the pause line is cut or missing: {texts:?}");
    assert!(!texts.iter().any(|text| text.contains("left")), "{texts:?}");
    let line = UiProbe::text_rect(&shapes, pause).expect("the pause line");
    let cancel = UiProbe::text_rect(&shapes, "Cancel").expect("the Cancel button");
    assert!(line.height() > TypeSize::Caption.line_height() * 1.5, "the pause line wraps: {line:?}");
    assert!(line.bottom() < cancel.top(), "the pause line stays above the footer: {line:?} {cancel:?}");

    let at = UiProbe::text_rect(&shapes, "Cancel").expect("the Cancel button").center();
    let mut probe = UiProbe::sized(1000.0, 800.0);
    assert_eq!(probe.click(at, |ui| RenderProgressCard::new(&view).show_card(ui)), Some(RenderProgressAction::Cancel));
}

#[test]
fn a_long_failure_text_scrolls_and_keeps_the_footer_on_screen() {
    let mut view = view(JobStatus::Failed, 0.5);
    view.time_left = words(&"The encoder failed: broken pipe. ".repeat(60));
    let mut probe = UiProbe::sized(1000.0, 400.0);
    probe.frame(Vec::new(), |ui| RenderProgressCard::new(&view).show(ui.ctx()));
    let (_, shapes) = probe.frame(Vec::new(), |ui| RenderProgressCard::new(&view).show(ui.ctx()));
    let close = UiProbe::text_rect(&shapes, "Close").expect("the Close button");
    let open = UiProbe::text_rect(&shapes, "Open queue").expect("the Open queue button");
    assert!(close.bottom() <= 400.0, "the footer is on screen: {close:?}");
    assert!(open.bottom() <= 400.0, "{open:?}");
}
