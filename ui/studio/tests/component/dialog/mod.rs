mod coverage_panel;
mod escape;
mod export_dialog;
mod rename_dialog;
mod render_progress;
mod render_queue;

use amml_studio_ui::component::dialog::{
    ExportDialogView, ExportFormat, ExportFrameRate, ExportResolution, JobStatus, RenderJobRow, RenderQueueView,
};
use amml_studio_ui::tokens::selector::SceneColor;

use crate::glyph::words;

// a queue of one job in `status` at `progress`
pub fn queue(status: JobStatus, progress: f32) -> RenderQueueView {
    RenderQueueView {
        subtitle: words("1 job Â· 1 to go"),
        paused: false,
        jobs: vec![RenderJobRow {
            name: words("Arrival"),
            meta: words("H.265 Â· 4K Â· 60 fps Â· 372 frames"),
            file: words("nether-trip_arrival_4k60.mp4"),
            scene: SceneColor::Lime,
            progress,
            status,
            detail: words("ETA 6 s"),
            note: words("Frame 106 / 372 Â· 45 fps"),
        }],
    }
}

// an export dialog for H.265 4K60 with nothing blocked
pub fn export() -> ExportDialogView {
    ExportDialogView {
        subtitle: words("Arrival · output 6.2 s"),
        format: ExportFormat::H265,
        resolution: ExportResolution::Uhd,
        frame_rate: ExportFrameRate::Sixty,
        quality: 0.75,
        quality_label: words("High"),
        pro_res_profile: None,
        h265_profile: None,
        sidecar: true,
        folder: words("C:\\Videos\\DejaVu\\"),
        file: words("nether-trip_arrival_4k60.mp4"),
        note: words("Not rendered yet."),
        unavailable: Vec::new(),
        render_blocked: None,
    }
}
