use amml_ui::gallery::GallerySheet;
use egui::Ui;

use super::{
    CaptureRailPage, CurvePlotPage, DialogsPage, EasingPopoverPage, EditLockBannerPage, GalleryState, GlyphsPage,
    HeroCardPage, InspectorPage, KeyHeaderPage, LibraryAppBarPage, PathMapPage, PlayerRowPage, ReplayRowPage,
    SceneCardPage, SceneSwitcherChipPage, SceneSwitcherListPage, SourceListPage, StatusBadgePage, StatusBarPage,
    TimelinePage, TimelineRowsPage, TimelineSamples, TimelineTracksPage, TopBarPage, TransportPage, ViewerChipsPage,
    ViewerPage,
};

/// One gallery page per Paper artboard group, in Paper order. Each component task adds its pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GalleryPage {
    Glyphs,
    StatusBadge,
    CurvePlot,
    TopBar,
    Transport,
    StatusBar,
    SceneSwitcherChip,
    Viewer,
    ViewerChips,
    PathMap,
    Timeline,
    Ruler,
    SceneBands,
    RealmRow,
    TrackHeaders,
    Lanes,
    BoxSelect,
    InspectorTabs,
    InspectorLocked,
    UndoHistory,
    EasingPopover,
    ExportDialog,
    ExportDialogProRes,
    ExportDialogH265Profile,
    RenderQueue,
    RenderQueueStates,
    RenderProgress,
    CoveragePanel,
    ConfirmDialog,
    RenameDialog,
    CaptureRail,
    LibraryAppBar,
    SceneCard,
    HeroCard,
    EditLockBanner,
    PlayerRow,
    KeyHeader,
    SceneSwitcherList,
    ReplayRow,
    SourceList,
}

impl GalleryPage {
    pub const ALL: [Self; 40] = [
        Self::Glyphs,
        Self::StatusBadge,
        Self::CurvePlot,
        Self::TopBar,
        Self::Transport,
        Self::StatusBar,
        Self::SceneSwitcherChip,
        Self::Viewer,
        Self::ViewerChips,
        Self::PathMap,
        Self::Timeline,
        Self::Ruler,
        Self::SceneBands,
        Self::RealmRow,
        Self::TrackHeaders,
        Self::Lanes,
        Self::BoxSelect,
        Self::InspectorTabs,
        Self::InspectorLocked,
        Self::UndoHistory,
        Self::EasingPopover,
        Self::ExportDialog,
        Self::ExportDialogProRes,
        Self::ExportDialogH265Profile,
        Self::RenderQueue,
        Self::RenderQueueStates,
        Self::RenderProgress,
        Self::CoveragePanel,
        Self::ConfirmDialog,
        Self::RenameDialog,
        Self::CaptureRail,
        Self::LibraryAppBar,
        Self::SceneCard,
        Self::HeroCard,
        Self::EditLockBanner,
        Self::PlayerRow,
        Self::KeyHeader,
        Self::SceneSwitcherList,
        Self::ReplayRow,
        Self::SourceList,
    ];

    /// The Paper artboard name, shown in the gallery's page list.
    pub fn name(self) -> &'static str {
        match self {
            Self::Glyphs => "Glyphs",
            Self::StatusBadge => "Status badge + Placed badge",
            Self::CurvePlot => "Curve plot",
            Self::TopBar => "Top bar",
            Self::Transport => "Transport",
            Self::StatusBar => "Status bar",
            Self::SceneSwitcherChip => "Scene switcher chip",
            Self::Viewer => "Viewer",
            Self::ViewerChips => "Viewer chips and overlays",
            Self::PathMap => "Path map",
            Self::Timeline => "Timeline · assembled",
            Self::Ruler => "Ruler",
            Self::SceneBands => "Scene bands",
            Self::RealmRow => "Realm row",
            Self::TrackHeaders => "Track headers",
            Self::Lanes => "Lanes",
            Self::BoxSelect => "Box select, multi-select, lock",
            Self::InspectorTabs => "Key active",
            Self::InspectorLocked => "Locked",
            Self::UndoHistory => "Undo history",
            Self::EasingPopover => "Easing popover",
            Self::ExportDialog => "Export dialog",
            Self::ExportDialogProRes => "Export dialog, ProRes 422",
            Self::ExportDialogH265Profile => "Export dialog, H.265 Main 10",
            Self::RenderQueue => "Render queue",
            Self::RenderQueueStates => "Render queue states",
            Self::RenderProgress => "Render progress",
            Self::CoveragePanel => "Coverage panel",
            Self::ConfirmDialog => "Confirm delete",
            Self::RenameDialog => "Rename",
            Self::CaptureRail => "Capture rail",
            Self::LibraryAppBar => "Library app bar",
            Self::SceneCard => "Scene card",
            Self::HeroCard => "Hero card",
            Self::EditLockBanner => "Edit lock banner",
            Self::PlayerRow => "Player row",
            Self::KeyHeader => "Key header",
            Self::SceneSwitcherList => "Scene switcher list",
            Self::ReplayRow => "Replay row",
            Self::SourceList => "Source list",
        }
    }

    /// The snapshot file stem.
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Glyphs => "glyphs",
            Self::StatusBadge => "status_badge",
            Self::CurvePlot => "curve_plot",
            Self::TopBar => "top_bar",
            Self::Transport => "transport",
            Self::StatusBar => "status_bar",
            Self::SceneSwitcherChip => "scene_switcher_chip",
            Self::Viewer => "viewer",
            Self::ViewerChips => "viewer_chips",
            Self::PathMap => "path_map",
            Self::Timeline => "timeline",
            Self::Ruler => "ruler",
            Self::SceneBands => "scene_bands",
            Self::RealmRow => "realm_row",
            Self::TrackHeaders => "track_headers",
            Self::Lanes => "lanes",
            Self::BoxSelect => "box_select",
            Self::InspectorTabs => "inspector_tabs",
            Self::InspectorLocked => "inspector_locked",
            Self::UndoHistory => "undo_history",
            Self::EasingPopover => "easing_popover",
            Self::ExportDialog => "export_dialog",
            Self::ExportDialogProRes => "export_dialog_pro_res",
            Self::ExportDialogH265Profile => "export_dialog_h265_profile",
            Self::RenderQueue => "render_queue",
            Self::RenderQueueStates => "render_queue_states",
            Self::RenderProgress => "render_progress",
            Self::CoveragePanel => "coverage_panel",
            Self::ConfirmDialog => "confirm_dialog",
            Self::RenameDialog => "rename_dialog",
            Self::CaptureRail => "capture_rail",
            Self::LibraryAppBar => "library_app_bar",
            Self::SceneCard => "scene_card",
            Self::HeroCard => "hero_card",
            Self::EditLockBanner => "edit_lock_banner",
            Self::PlayerRow => "player_row",
            Self::KeyHeader => "key_header",
            Self::SceneSwitcherList => "scene_switcher_list",
            Self::ReplayRow => "replay_row",
            Self::SourceList => "source_list",
        }
    }

    /// The width of the page's Paper artboard.
    pub fn width(self) -> f32 {
        match self {
            // these pages use the amml-ui sheet chrome
            Self::Glyphs
            | Self::StatusBadge
            | Self::CurvePlot
            | Self::SceneSwitcherChip
            | Self::LibraryAppBar
            | Self::SceneCard
            | Self::HeroCard
            | Self::EditLockBanner
            | Self::PlayerRow
            | Self::KeyHeader
            | Self::SceneSwitcherList
            | Self::ReplayRow
            | Self::SourceList => GallerySheet::WIDTH,
            Self::TopBar => TopBarPage::WIDTH,
            Self::Transport => TransportPage::WIDTH,
            Self::StatusBar => StatusBarPage::WIDTH,
            Self::Viewer => ViewerPage::WIDTH,
            Self::ViewerChips => ViewerChipsPage::WIDTH,
            Self::PathMap => PathMapPage::WIDTH,
            Self::Timeline
            | Self::Ruler
            | Self::SceneBands
            | Self::RealmRow
            | Self::TrackHeaders
            | Self::Lanes
            | Self::BoxSelect => TimelineSamples::WIDTH,
            Self::InspectorTabs | Self::InspectorLocked | Self::UndoHistory => InspectorPage::WIDTH,
            Self::EasingPopover => EasingPopoverPage::WIDTH,
            Self::ExportDialog | Self::ExportDialogProRes | Self::ExportDialogH265Profile => DialogsPage::EXPORT,
            Self::RenderQueue | Self::RenderQueueStates => DialogsPage::QUEUE,
            Self::RenderProgress => DialogsPage::PROGRESS,
            Self::CoveragePanel => DialogsPage::COVERAGE,
            Self::ConfirmDialog => DialogsPage::CONFIRM,
            Self::RenameDialog => DialogsPage::RENAME,
            Self::CaptureRail => CaptureRailPage::WIDTH,
        }
    }

    pub fn show(self, ui: &mut Ui, state: &mut GalleryState) {
        match self {
            Self::Glyphs => GlyphsPage::show(ui, state),
            Self::StatusBadge => StatusBadgePage::show(ui, state),
            Self::CurvePlot => CurvePlotPage::show(ui, state),
            Self::TopBar => TopBarPage::show(ui, state),
            Self::Transport => TransportPage::show(ui, state),
            Self::StatusBar => StatusBarPage::show(ui, state),
            Self::SceneSwitcherChip => SceneSwitcherChipPage::show(ui, state),
            Self::Viewer => ViewerPage::show(ui, state),
            Self::ViewerChips => ViewerChipsPage::show(ui, state),
            Self::PathMap => PathMapPage::show(ui, state),
            Self::Timeline => TimelinePage::show(ui, state),
            Self::Ruler => TimelineRowsPage::ruler(ui, state),
            Self::SceneBands => TimelineRowsPage::scene_bands(ui, state),
            Self::RealmRow => TimelineRowsPage::realm_row(ui, state),
            Self::TrackHeaders => TimelineTracksPage::track_headers(ui, state),
            Self::Lanes => TimelineTracksPage::lanes(ui, state),
            Self::BoxSelect => TimelineTracksPage::box_select(ui, state),
            Self::InspectorTabs => InspectorPage::tabs(ui, state),
            Self::InspectorLocked => InspectorPage::locked(ui, state),
            Self::UndoHistory => InspectorPage::undo_history(ui, state),
            Self::EasingPopover => EasingPopoverPage::show(ui, state),
            Self::ExportDialog => DialogsPage::export(ui, state),
            Self::ExportDialogProRes => DialogsPage::export_pro_res(ui, state),
            Self::ExportDialogH265Profile => DialogsPage::export_h265_profile(ui, state),
            Self::RenderQueue => DialogsPage::queue(ui, state),
            Self::RenderQueueStates => DialogsPage::queue_states(ui, state),
            Self::RenderProgress => DialogsPage::progress(ui, state),
            Self::CoveragePanel => DialogsPage::coverage(ui, state),
            Self::ConfirmDialog => DialogsPage::confirm(ui, state),
            Self::RenameDialog => DialogsPage::rename(ui, state),
            Self::CaptureRail => CaptureRailPage::show(ui, state),
            Self::LibraryAppBar => LibraryAppBarPage::show(ui, state),
            Self::SceneCard => SceneCardPage::show(ui, state),
            Self::HeroCard => HeroCardPage::show(ui, state),
            Self::EditLockBanner => EditLockBannerPage::show(ui, state),
            Self::PlayerRow => PlayerRowPage::show(ui, state),
            Self::KeyHeader => KeyHeaderPage::show(ui, state),
            Self::SceneSwitcherList => SceneSwitcherListPage::show(ui, state),
            Self::ReplayRow => ReplayRowPage::show(ui, state),
            Self::SourceList => SourceListPage::show(ui, state),
        }
    }
}
