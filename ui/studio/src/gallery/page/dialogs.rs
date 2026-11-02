use egui::{Id, Ui, vec2};

use super::{BarRows, TimelineSamples, ViewerSamples};
use crate::component::dialog::{
    ConfirmDialog, ConfirmDialogView, CoverageFilter, CoverageGroup, CoveragePanel, CoveragePanelView, CoverageRow,
    ExportDialog, ExportDialogView, ExportFormat, ExportFrameRate, ExportH265Profile, ExportProResProfile, ExportResolution, JobStatus, RenameDialog,
    RenderJobRow, RenderProgressCard, RenderProgressView, RenderQueue, RenderQueueView,
};
use crate::gallery::GalleryState;
use crate::id::StudioUiId;
use crate::tokens::selector::{SceneColor, StatusColor};

/// Paper: Menus + Dialogs, the dialog sheets: `Export dialog` (`971-0`), `Render queue` (`9A9-0`, `9D1-0`),
/// `Render progress — rendering, failed`, `Coverage panel` (`9GC-0`, `9H2-0`, `9HR-0`), `Confirm delete` (`9PB-0`) and `Rename` (`9QD-0`). Each card is
/// drawn where the page puts it, with no scrim.
pub struct DialogsPage;

impl DialogsPage {
    /// Paper: the sheet widths.
    pub const EXPORT: f32 = 780.0;
    pub const QUEUE: f32 = 900.0;
    pub const PROGRESS: f32 = 1120.0;
    pub const COVERAGE: f32 = 1040.0;
    pub const CONFIRM: f32 = 1100.0;
    pub const RENAME: f32 = 1580.0;
    // sheet chrome: the inset of the cards and the gap between side-by-side cards
    const INSET: f32 = 40.0;
    const GAP: f32 = 40.0;

    fn words(text: &str) -> amml_i18n::text::Text {
        TimelineSamples::words(text)
    }

    // a caption, then `body` 40 in from the page's left
    fn sheet(ui: &mut Ui, caption: &str, body: impl FnOnce(&mut Ui)) {
        ViewerSamples::row(ui, caption, |ui| {
            ui.horizontal_top(|ui| {
                ui.add_space(Self::INSET);
                ui.spacing_mut().item_spacing = vec2(Self::GAP, 0.0);
                body(ui);
            });
        });
    }

    pub fn export(ui: &mut Ui, _state: &mut GalleryState) {
        let view = ExportDialogView {
            subtitle: Self::words("Arrival · 00:00:28.80 → 00:00:37.40 · output 6.2 s"),
            format: ExportFormat::H265,
            resolution: ExportResolution::Uhd,
            frame_rate: ExportFrameRate::Sixty,
            quality: 0.75,
            quality_label: Self::words("High"),
            pro_res_profile: None,
            h265_profile: None,
            sidecar: true,
            folder: Self::words("C:\\Users\\…\\Videos\\DejaVu\\"),
            file: Self::words("nether-trip_arrival_4k60.mp4"),
            note: Self::words("Not rendered yet. Video only: audio is not exported."),
            unavailable: Vec::new(),
            render_blocked: None,
        };
        BarRows::page(ui, |ui| {
            Self::sheet(ui, "EXPORT SCENE", |ui| {
                ui.vertical(|ui| ExportDialog::new(&view).show_card(ui));
            });
        });
    }

    pub fn export_pro_res(ui: &mut Ui, _state: &mut GalleryState) {
        let view = ExportDialogView {
            subtitle: Self::words("Arrival · 00:00:28.80 → 00:00:37.40 · output 6.2 s"),
            format: ExportFormat::ProRes422,
            resolution: ExportResolution::Uhd,
            frame_rate: ExportFrameRate::Sixty,
            quality: 0.75,
            quality_label: Self::words("High"),
            pro_res_profile: Some(ExportProResProfile::Hq),
            h265_profile: None,
            sidecar: true,
            folder: Self::words("C:\\Users\\…\\Videos\\DejaVu\\"),
            file: Self::words("nether-trip_arrival_4k60.mov"),
            note: Self::words("Not rendered yet. Video only: audio is not exported."),
            unavailable: Vec::new(),
            render_blocked: None,
        };
        BarRows::page(ui, |ui| {
            Self::sheet(ui, "EXPORT SCENE, PRORES 422", |ui| {
                ui.vertical(|ui| ExportDialog::new(&view).show_card(ui));
            });
        });
    }

    pub fn export_h265_profile(ui: &mut Ui, _state: &mut GalleryState) {
        let view = ExportDialogView {
            subtitle: Self::words("Arrival · 00:00:28.80 → 00:00:37.40 · output 6.2 s"),
            format: ExportFormat::H265,
            resolution: ExportResolution::Uhd,
            frame_rate: ExportFrameRate::Sixty,
            quality: 0.75,
            quality_label: Self::words("High"),
            pro_res_profile: None,
            h265_profile: Some(ExportH265Profile::Main10),
            sidecar: true,
            folder: Self::words("C:\\Users\\…\\Videos\\DejaVu\\"),
            file: Self::words("nether-trip_arrival_4k60.mp4"),
            note: Self::words("Not rendered yet. Video only: audio is not exported."),
            unavailable: Vec::new(),
            render_blocked: None,
        };
        BarRows::page(ui, |ui| {
            Self::sheet(ui, "EXPORT SCENE, H.265 MAIN 10", |ui| {
                ui.vertical(|ui| ExportDialog::new(&view).show_card(ui));
            });
        });
    }

    fn job(name: &str, meta: &str, file: &str, scene: SceneColor, status: JobStatus, detail: &str) -> RenderJobRow {
        RenderJobRow {
            name: Self::words(name),
            meta: Self::words(meta),
            file: Self::words(file),
            scene,
            progress: 0.28,
            status,
            detail: Self::words(detail),
            note: Self::words("Frame 106 / 372 · 45 fps"),
        }
    }

    pub fn queue(ui: &mut Ui, _state: &mut GalleryState) {
        let arrival = || {
            Self::job(
                "Arrival",
                "H.265 · 4K · 60 fps · 372 frames",
                "nether-trip_arrival_4k60.mp4",
                SceneColor::Lime,
                JobStatus::Rendering,
                "ETA 6 s",
            )
        };
        let portal = || {
            Self::job(
                "Portal run",
                "H.264 · 1080p · 30 fps · 426 frames",
                "nether-trip_portal-run_1080p30.mp4",
                SceneColor::Pink,
                JobStatus::Queued,
                "#1 in line",
            )
        };
        let rendering = RenderQueueView {
            subtitle: Self::words("4 jobs · 2 to go · about 14 s left"),
            paused: false,
            jobs: vec![
                Self::job(
                    "Blaze farm",
                    "H.265 · 4K · 60 fps · 480 frames",
                    "nether-trip_blaze-farm_4k60.mp4",
                    SceneColor::Peach,
                    JobStatus::Failed,
                    "Scene has no Camera keys",
                ),
                arrival(),
                portal(),
                Self::job(
                    "Multi-angle B",
                    "H.265 · 1440p · 60 fps · 324 frames",
                    "nether-trip_multi-angle-b_1440p60.mp4",
                    SceneColor::Periwinkle,
                    JobStatus::Done,
                    "",
                ),
            ],
        };
        let paused = RenderQueueView {
            subtitle: Self::words("3 jobs · 2 to go · about 14 s left · PAUSED"),
            paused: true,
            jobs: vec![
                arrival(),
                portal(),
                Self::job(
                    "Fortress",
                    "H.265 · 4K · 60 fps · 960 frames",
                    "nether-trip_fortress_4k60.mp4",
                    SceneColor::Orchid,
                    JobStatus::Cancelled,
                    "",
                ),
            ],
        };
        BarRows::page(ui, |ui| {
            Self::sheet(ui, "RENDERING", |ui| {
                ui.push_id(0, |ui| ui.vertical(|ui| RenderQueue::new(&rendering).show_card(ui)));
            });
            Self::sheet(ui, "PAUSED · CANCELLED JOB", |ui| {
                ui.push_id(1, |ui| ui.vertical(|ui| RenderQueue::new(&paused).show_card(ui)));
            });
        });
    }

    /// Paper: `Dialog / Render queue — preparing, finishing, reorder menu`. The job menu is the studio's.
    pub fn queue_states(ui: &mut Ui, _state: &mut GalleryState) {
        let preparing = |status, detail| {
            Self::job(
                "Arrival",
                "H.265 · 4K · 60 fps · 372 frames",
                "nether-trip_arrival_4k60.mp4",
                SceneColor::Lime,
                status,
                detail,
            )
        };
        let reorder = RenderQueueView {
            subtitle: Self::words("3 jobs · 1 to go · about 14 s left"),
            paused: false,
            jobs: vec![
                preparing(JobStatus::Preparing, "Getting ready"),
                Self::job(
                    "Deleted scene",
                    "H.264 · 1080p · 30 fps · 426 frames",
                    "nether-trip_deleted-scene_1080p30.mp4",
                    SceneColor::Pink,
                    JobStatus::Failed,
                    "The scene is gone.",
                ),
                Self::job(
                    "Fortress",
                    "H.265 · 4K · 60 fps · 960 frames",
                    "nether-trip_fortress_4k60.mp4",
                    SceneColor::Orchid,
                    JobStatus::Queued,
                    "#1 in line",
                ),
            ],
        };
        let finishing = RenderQueueView {
            subtitle: Self::words("3 jobs · 1 to go · about 14 s left"),
            paused: false,
            jobs: vec![preparing(JobStatus::Finishing, "Writing the file")],
        };
        BarRows::page(ui, |ui| {
            Self::sheet(ui, "PREPARING · FAILED · QUEUED", |ui| {
                ui.push_id(0, |ui| ui.vertical(|ui| RenderQueue::new(&reorder).show_card(ui)));
            });
            Self::sheet(ui, "FINISHING", |ui| {
                ui.push_id(1, |ui| ui.vertical(|ui| RenderQueue::new(&finishing).show_card(ui)));
            });
        });
    }

    fn row(id: &str, label: &str, status: StatusColor, area: &str, go: bool) -> CoverageRow {
        CoverageRow { id: Self::words(id), label: Self::words(label), status, area: Self::words(area), go }
    }

    fn coverage_view(filter: CoverageFilter) -> CoveragePanelView {
        let group = |title: &str, rows: Vec<CoverageRow>| CoverageGroup { title: Self::words(title), rows };
        CoveragePanelView {
            subtitle: Self::words(
                "Status of each capability in the real mod (feature inventory, 2026-10-21). Click an item to go to it.",
            ),
            filter,
            groups: vec![
                group(
                    "A. Library and open replay",
                    vec![
                        Self::row("A1", "Main-menu \"Open Replay\" button", StatusColor::Works, "Library", true),
                        Self::row("A2", "Replay list", StatusColor::Works, "Library", true),
                        Self::row("A11", "Replay details panel", StatusColor::Mockup, "Library", true),
                        Self::row("A14", "Rename a replay", StatusColor::Missing, "Library", true),
                        Self::row("A15", "egui launcher window", StatusColor::Planned, "Library", true),
                    ],
                ),
                group(
                    "B. Joining and viewer",
                    vec![
                        Self::row("B1", "Join the replay by itself", StatusColor::Works, "Library", true),
                        Self::row("B12", "Last rebuild warning", StatusColor::Engine, "Editor", true),
                        Self::row("B13", "Composition guides and viewer chips", StatusColor::Planned, "Editor", true),
                    ],
                ),
                group(
                    "D. Seek and rewind",
                    vec![Self::row("D5", "Rebuild progress", StatusColor::Missing, "Editor", true)],
                ),
                group(
                    "K. FOV",
                    vec![
                        Self::row("K1", "Scene FOV", StatusColor::Works, "Scene tab", true),
                        Self::row("K4", "Per-frame FOV", StatusColor::Works, "not in lab", false),
                        Self::row("K5", "FOV inside the camera key", StatusColor::Planned, "Key tab", true),
                    ],
                ),
            ],
        }
    }

    pub fn progress(ui: &mut Ui, _state: &mut GalleryState) {
        let rendering = RenderProgressView {
            title: Self::words("Rendering “Arrival”"),
            file: Self::words("nether-trip_arrival_4k60.mp4"),
            status: JobStatus::Rendering,
            running: true,
            progress: 106.0 / 372.0,
            frames: Self::words("Frame 106 of 372"),
            time_left: Self::words("About 6 s left"),
        };
        let failed = RenderProgressView {
            title: Self::words("The render of “Arrival” failed"),
            file: Self::words("nether-trip_arrival_4k60.mp4"),
            status: JobStatus::Failed,
            running: false,
            progress: 212.0 / 372.0,
            frames: Self::words("Frame 212 of 372"),
            time_left: Self::words("The encoder failed at frame 212: broken pipe"),
        };
        BarRows::page(ui, |ui| {
            Self::sheet(ui, "RENDERING", |ui| {
                ui.push_id(0, |ui| ui.vertical(|ui| RenderProgressCard::new(&rendering).show_card(ui)));
            });
            Self::sheet(ui, "FAILED", |ui| {
                ui.push_id(1, |ui| ui.vertical(|ui| RenderProgressCard::new(&failed).show_card(ui)));
            });
        });
    }

    pub fn coverage(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            let states = [
                ("ALL ITEMS", CoverageFilter::All, ""),
                ("MISSING FILTER", CoverageFilter::Status(StatusColor::Missing), ""),
                ("SEARCH \"FOV\"", CoverageFilter::All, "fov"),
            ];
            for (index, (caption, filter, search)) in states.into_iter().enumerate() {
                let view = Self::coverage_view(filter);
                let mut search = search.to_owned();
                Self::sheet(ui, caption, |ui| {
                    let field = Id::unique(StudioUiId::CoverageSearch).with(index);
                    ui.push_id(index, |ui| {
                        ui.vertical(|ui| CoveragePanel::new(&view, &mut search).id(field).show_card(ui))
                    });
                });
            }
        });
    }

    pub fn confirm(ui: &mut Ui, _state: &mut GalleryState) {
        let cards = [
            ("Delete “Portal run”?", "The scene and its 11 keys go away. Undo brings them back.", "Delete scene"),
            ("Remove the Camera track?", "It has 6 keys.", "Remove track"),
        ];
        BarRows::page(ui, |ui| {
            Self::sheet(ui, "DELETE SCENE · REMOVE TRACK", |ui| {
                for (index, (title, body, confirm)) in cards.into_iter().enumerate() {
                    let view = ConfirmDialogView {
                        title: Self::words(title),
                        subtitle: Self::words("Undo can bring it back"),
                        body: Self::words(body),
                        confirm: Self::words(confirm),
                    };
                    ui.push_id(index, |ui| ui.vertical(|ui| ConfirmDialog::new(&view).show_card(ui)));
                }
            });
        });
    }

    pub fn rename(ui: &mut Ui, _state: &mut GalleryState) {
        let cards = [("Rename scene", "Portal run"), ("Rename bookmark", "portal"), ("Rename easing preset", "")];
        BarRows::page(ui, |ui| {
            Self::sheet(ui, "SCENE · BOOKMARK · PRESET", |ui| {
                for (index, (title, value)) in cards.into_iter().enumerate() {
                    let title = Self::words(title);
                    let mut value = value.to_owned();
                    let field = Id::unique(StudioUiId::RenameField).with(index);
                    ui.push_id(index, |ui| {
                        ui.vertical(|ui| RenameDialog::new(&title, &mut value).id(field).show_card(ui))
                    });
                }
            });
        });
    }
}
