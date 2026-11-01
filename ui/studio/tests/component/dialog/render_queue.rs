use amml_studio_ui::component::dialog::{JobStatus, RenderQueue, RenderQueueAction, RenderQueueView};
use amml_ui::icon::{Icon, IconShape};
use amml_ui::tokens::{ColorToken, color};
use egui::Shape;

use super::queue;
use crate::glyph::words;
use crate::support::UiProbe;

// the width of the accent progress fill
fn bar(progress: f32) -> f32 {
    let view = queue(JobStatus::Rendering, progress);
    let shapes = UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderQueue::new(&view).show_card(ui));
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
fn progress_is_clamped() {
    let full = bar(1.0);
    assert!(full > 0.0);
    assert_eq!(bar(f32::NAN), 0.0);
    assert_eq!(bar(-0.5), 0.0);
    assert_eq!(bar(1.5), full);
}

#[test]
fn queue_with_no_jobs() {
    let view = RenderQueueView { subtitle: words("No jobs"), paused: false, jobs: Vec::new() };
    let texts = UiProbe::texts(&UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderQueue::new(&view).show_card(ui)));
    assert!(texts.iter().any(|text| text == "No render jobs"), "{texts:?}");
}

fn one(status: JobStatus, detail: &str) -> RenderQueueView {
    let mut view = queue(status, 0.0);
    view.jobs[0].detail = words(detail);
    view
}

fn glyph(icon: Icon) -> String {
    match icon.shape() {
        IconShape::Font(code) => code.to_string(),
        IconShape::Drawn(_) => panic!("{icon:?} is drawn"),
    }
}

#[test]
fn the_trailing_button_cancels_a_job_that_is_not_over_and_removes_one_that_is() {
    for (status, icon, action) in [
        (JobStatus::Queued, Icon::Stop, RenderQueueAction::Cancel(0)),
        (JobStatus::Preparing, Icon::Stop, RenderQueueAction::Cancel(0)),
        (JobStatus::Rendering, Icon::Stop, RenderQueueAction::Cancel(0)),
        (JobStatus::Finishing, Icon::Stop, RenderQueueAction::Cancel(0)),
        (JobStatus::Done, Icon::Close, RenderQueueAction::Remove(0)),
        (JobStatus::Failed, Icon::Close, RenderQueueAction::Remove(0)),
        (JobStatus::Cancelled, Icon::Close, RenderQueueAction::Remove(0)),
    ] {
        let view = one(status, "");
        let shapes = UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderQueue::new(&view).show_card(ui));
        // the header's close mark is the same glyph: the row's button is the lower one
        let at = shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Text(text) if text.galley.text() == glyph(icon) => Some(text.visual_bounding_rect().center()),
                _ => None,
            })
            .max_by(|a, b| a.y.total_cmp(&b.y))
            .unwrap_or_else(|| panic!("{status:?}: no {icon:?} button"));
        let mut probe = UiProbe::sized(1000.0, 800.0);
        assert_eq!(probe.click(at, |ui| RenderQueue::new(&view).show_card(ui)), Some(action), "{status:?}");
    }
}

#[test]
fn a_right_click_on_a_row_asks_for_the_job_menu() {
    let view = one(JobStatus::Queued, "#1 in line");
    let shapes = UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderQueue::new(&view).show_card(ui));
    let at = UiProbe::text_rect(&shapes, "Arrival").expect("the job name").center();
    let mut probe = UiProbe::sized(1000.0, 800.0);
    let action = probe.secondary_click(at, |ui| RenderQueue::new(&view).show_card(ui));
    assert!(matches!(action, Some(RenderQueueAction::OpenJobMenu(0, _))), "{action:?}");
}

#[test]
fn preparing_and_finishing_show_their_pill_and_detail() {
    for (status, label, detail) in
        [(JobStatus::Preparing, "Preparing", "Getting ready"), (JobStatus::Finishing, "Finishing", "Writing the file")]
    {
        let view = one(status, detail);
        let texts = UiProbe::texts(&UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderQueue::new(&view).show_card(ui)));
        for shown in [label, detail] {
            assert!(texts.iter().any(|text| text == shown), "{shown} is missing: {texts:?}");
        }
    }
}

#[test]
fn a_failed_job_shows_its_reason_under_the_pill() {
    let view = one(JobStatus::Failed, "The scene is gone.");
    let shapes = UiProbe::sized(1000.0, 800.0).shapes(|ui| RenderQueue::new(&view).show_card(ui));
    let pill = UiProbe::text_rect(&shapes, "Failed").expect("the pill");
    let reason = UiProbe::text_rect(&shapes, "The scene is gone.").expect("the reason");
    let retry = UiProbe::text_rect(&shapes, "Retry").expect("Retry");
    assert!(reason.top() > pill.bottom(), "reason {reason:?} under the pill {pill:?}");
    // the pill's word is inset in the pill; the reason starts at the pill's edge
    assert!(reason.left() < pill.left() && pill.left() - reason.left() < 24.0, "the reason starts under the pill");
    assert!((retry.center().y - pill.center().y).abs() < 4.0, "Retry is on the pill's line");
}
