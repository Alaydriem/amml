use amml_i18n::text::Text;

use super::JobStatus;
use crate::tokens::selector::SceneColor;

/// One render job: its scene name, the settings line (as in "H.265 · 4K · 60 fps · 372 frames"), the file name, the
/// scene's colour, its progress from 0 to 1 (any other value is clamped; not a number shows as 0), its status,
/// `detail` beside the status (an ETA, an error, "#1 in line") and `note` under a rendering job's bar.
#[derive(Debug, Clone)]
pub struct RenderJobRow {
    pub name: Text,
    pub meta: Text,
    pub file: Text,
    pub scene: SceneColor,
    pub progress: f32,
    pub status: JobStatus,
    pub detail: Text,
    pub note: Text,
}
