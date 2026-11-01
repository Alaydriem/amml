use amml_i18n::text::Text;

use crate::component::dialog::JobStatus;

/// What the render progress card shows: the title (as in "Rendering “Arrival”"), the output file, the job status,
/// the progress from 0 to 1, the frame count (as in "Frame 106 of 372"), and the line under the bar (the time
/// left while it runs, else the result or why it failed).
#[derive(Debug, Clone)]
pub struct RenderProgressView {
    pub title: Text,
    pub file: Text,
    pub status: JobStatus,
    /// The render runs: the card offers Cancel, else Close.
    pub running: bool,
    pub progress: f32,
    pub frames: Text,
    pub time_left: Text,
}
