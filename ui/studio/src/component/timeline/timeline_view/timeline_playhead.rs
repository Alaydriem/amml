use amml_i18n::text::Text;

/// The playhead: its time in seconds, the time as the caller writes it in the head (as in "32.95"), and whether the
/// scene camera drives (the playhead then takes `--color-scene-camera`).
#[derive(Debug, Clone)]
pub struct TimelinePlayhead {
    pub seconds: f64,
    pub label: Text,
    pub scene_drives: bool,
}
