use amml_studio_ui::component::inspector::{UndoHistoryAction, UndoHistoryPopover, UndoHistoryView};

use crate::glyph::words;
use crate::support::UiProbe;

fn six() -> UndoHistoryView {
    let steps = ["Easing", "Move keys", "Edit key", "Add key", "Scene settings", "Rename scene"];
    UndoHistoryView { steps: steps.iter().map(|step| words(step)).collect() }
}

#[test]
fn undo_history_jumps_to_an_entry() {
    let view = six();
    let mut probe = UiProbe::new(400.0);
    let shapes = probe.shapes(|ui| UndoHistoryPopover::new(&view).show(ui));
    let third = UiProbe::text_rect(&shapes, "3. Edit key").expect("the third step is painted");
    let picked = probe.click(third.center(), |ui| UndoHistoryPopover::new(&view).show(ui));
    assert_eq!(picked, Some(UndoHistoryAction::JumpTo(2)));
}

#[test]
fn long_undo_history_scrolls_and_reaches_both_ends() {
    let view = UndoHistoryView { steps: (0..200).map(|index| words(&format!("Step {index}"))).collect() };
    let mut probe = UiProbe::new(400.0);
    let show = |ui: &mut egui::Ui| (UndoHistoryPopover::new(&view).show(ui), ui.min_rect());
    let ((_, used), shapes) = {
        probe.frame(Vec::new(), show);
        probe.frame(Vec::new(), show)
    };
    assert!(used.height() <= 400.0, "the popover runs off the screen: {used:?}");
    let newest = UiProbe::text_rect(&shapes, "200. Step 199").expect("the newest step shows when it opens");
    assert_eq!(probe.click(newest.center(), show).0, Some(UndoHistoryAction::JumpTo(199)));
    probe.wheel(newest.center(), egui::vec2(0.0, 20000.0), show);
    let shapes = probe.shapes(show);
    let oldest = UiProbe::text_rect(&shapes, "1. Step 0").expect("the oldest step shows after a scroll up");
    assert_eq!(probe.click(oldest.center(), show).0, Some(UndoHistoryAction::JumpTo(0)));
}

#[test]
fn undo_history_with_no_entries() {
    let view = UndoHistoryView { steps: Vec::new() };
    let texts = UiProbe::texts(&UiProbe::new(400.0).shapes(|ui| UndoHistoryPopover::new(&view).show(ui)));
    assert_eq!(texts, ["Undo history", "Nothing to undo"]);
}
