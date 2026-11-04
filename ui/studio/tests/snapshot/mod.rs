mod page_snapshot;

use std::path::Path;

use amml_studio_ui::gallery::GalleryPage;
use page_snapshot::PageSnapshot;

#[test]
fn every_gallery_page_has_a_snapshot() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("snapshots");
    let missing: Vec<&str> = GalleryPage::ALL
        .iter()
        .map(|page| page.file_name())
        .filter(|name| !folder.join(format!("{name}.png")).is_file())
        .collect();
    assert!(missing.is_empty(), "no snapshot for {missing:?}: run `mise run amml-studio-ui:snapshots`");
}

#[test]
fn snapshot_glyphs() {
    PageSnapshot::check(GalleryPage::Glyphs);
}

#[test]
fn snapshot_status_badge() {
    PageSnapshot::check(GalleryPage::StatusBadge);
}

#[test]
fn snapshot_curve_plot() {
    PageSnapshot::check(GalleryPage::CurvePlot);
}

#[test]
fn snapshot_top_bar() {
    PageSnapshot::check(GalleryPage::TopBar);
}

#[test]
fn snapshot_transport() {
    PageSnapshot::check(GalleryPage::Transport);
}

#[test]
fn snapshot_status_bar() {
    PageSnapshot::check(GalleryPage::StatusBar);
}

#[test]
fn snapshot_scene_switcher_chip() {
    PageSnapshot::check(GalleryPage::SceneSwitcherChip);
}

#[test]
fn snapshot_viewer() {
    PageSnapshot::check(GalleryPage::Viewer);
}

#[test]
fn snapshot_viewer_chips() {
    PageSnapshot::check(GalleryPage::ViewerChips);
}

#[test]
fn snapshot_path_map() {
    PageSnapshot::check(GalleryPage::PathMap);
}

#[test]
fn snapshot_timeline() {
    PageSnapshot::check(GalleryPage::Timeline);
}

#[test]
fn snapshot_ruler() {
    PageSnapshot::check(GalleryPage::Ruler);
}

#[test]
fn snapshot_scene_bands() {
    PageSnapshot::check(GalleryPage::SceneBands);
}

#[test]
fn snapshot_realm_row() {
    PageSnapshot::check(GalleryPage::RealmRow);
}

#[test]
fn snapshot_track_headers() {
    PageSnapshot::check(GalleryPage::TrackHeaders);
}

#[test]
fn snapshot_lanes() {
    PageSnapshot::check(GalleryPage::Lanes);
}

#[test]
fn snapshot_box_select() {
    PageSnapshot::check(GalleryPage::BoxSelect);
}

#[test]
fn snapshot_inspector_tabs() {
    PageSnapshot::check(GalleryPage::InspectorTabs);
}

#[test]
fn snapshot_inspector_locked() {
    PageSnapshot::check(GalleryPage::InspectorLocked);
}

#[test]
fn snapshot_undo_history() {
    PageSnapshot::check(GalleryPage::UndoHistory);
}

#[test]
fn snapshot_easing_popover() {
    PageSnapshot::check(GalleryPage::EasingPopover);
}

#[test]
fn snapshot_export_dialog() {
    PageSnapshot::check(GalleryPage::ExportDialog);
}

#[test]
fn snapshot_export_dialog_pro_res() {
    PageSnapshot::check(GalleryPage::ExportDialogProRes);
}

#[test]
fn snapshot_export_dialog_h265_profile() {
    PageSnapshot::check(GalleryPage::ExportDialogH265Profile);
}

#[test]
fn snapshot_render_queue() {
    PageSnapshot::check(GalleryPage::RenderQueue);
}

#[test]
fn snapshot_render_queue_states() {
    PageSnapshot::check(GalleryPage::RenderQueueStates);
}

#[test]
fn snapshot_render_progress() {
    PageSnapshot::check(GalleryPage::RenderProgress);
}

#[test]
fn snapshot_coverage_panel() {
    PageSnapshot::check(GalleryPage::CoveragePanel);
}

#[test]
fn snapshot_confirm_dialog() {
    PageSnapshot::check(GalleryPage::ConfirmDialog);
}

#[test]
fn snapshot_rename_dialog() {
    PageSnapshot::check(GalleryPage::RenameDialog);
}

#[test]
fn snapshot_capture_rail() {
    PageSnapshot::check(GalleryPage::CaptureRail);
}

#[test]
fn snapshot_library_app_bar() {
    PageSnapshot::check(GalleryPage::LibraryAppBar);
}

#[test]
fn snapshot_scene_card() {
    PageSnapshot::check(GalleryPage::SceneCard);
}

#[test]
fn snapshot_hero_card() {
    PageSnapshot::check(GalleryPage::HeroCard);
}

#[test]
fn snapshot_edit_lock_banner() {
    PageSnapshot::check(GalleryPage::EditLockBanner);
}

#[test]
fn snapshot_player_row() {
    PageSnapshot::check(GalleryPage::PlayerRow);
}

#[test]
fn snapshot_key_header() {
    PageSnapshot::check(GalleryPage::KeyHeader);
}

#[test]
fn snapshot_scene_switcher_list() {
    PageSnapshot::check(GalleryPage::SceneSwitcherList);
}

#[test]
fn snapshot_replay_row() {
    PageSnapshot::check(GalleryPage::ReplayRow);
}

#[test]
fn snapshot_source_list() {
    PageSnapshot::check(GalleryPage::SourceList);
}
