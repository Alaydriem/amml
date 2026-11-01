use amml_studio_ui::component::dialog::{
    ConfirmAction, ConfirmDialog, ConfirmDialogView, CoverageFilter, CoveragePanel, CoveragePanelAction,
    CoveragePanelView, ExportDialog, ExportDialogAction, JobStatus, RenameAction, RenameDialog, RenderQueue,
    RenderQueueAction,
};
use egui::{Key, Ui};

use super::{export, queue};
use crate::glyph::words;
use crate::support::UiProbe;

// shows a dialog for a frame, then a frame with Escape pressed; returns the second frame's action
fn escape<T>(mut show: impl FnMut(&mut Ui) -> Option<T>) -> Option<T> {
    let mut probe = UiProbe::sized(1280.0, 900.0);
    probe.frame(Vec::new(), &mut show);
    probe.frame(vec![UiProbe::key(Key::Escape)], &mut show).0
}

#[test]
fn escape_cancels_each_dialog() {
    let export = export();
    assert_eq!(escape(|ui| ExportDialog::new(&export).show(ui.ctx())), Some(ExportDialogAction::Cancel));
    let jobs = queue(JobStatus::Queued, 0.0);
    assert_eq!(escape(|ui| RenderQueue::new(&jobs).show(ui.ctx())), Some(RenderQueueAction::Close));
    let coverage = CoveragePanelView { subtitle: words("Coverage"), filter: CoverageFilter::All, groups: Vec::new() };
    let mut search = String::new();
    assert_eq!(
        escape(|ui| CoveragePanel::new(&coverage, &mut search).show(ui.ctx())),
        Some(CoveragePanelAction::Close)
    );
    let confirm = ConfirmDialogView {
        title: words("Delete “Portal run”?"),
        subtitle: words("Undo can bring it back"),
        body: words("The scene and its 11 keys go away."),
        confirm: words("Delete scene"),
    };
    assert_eq!(escape(|ui| ConfirmDialog::new(&confirm).show(ui.ctx())), Some(ConfirmAction::Cancel));
    let title = words("Rename scene");
    let mut value = "Portal run".to_owned();
    assert_eq!(escape(|ui| RenameDialog::new(&title, &mut value).show(ui.ctx())), Some(RenameAction::Cancel));
}
