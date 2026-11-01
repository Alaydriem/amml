use amml_i18n::text::Text;

use super::{ExportChoice, ExportFormat, ExportFrameRate, ExportResolution};

/// What the export dialog shows. The words beside the controls come from the caller: the quality name (as in
/// "High"), the output folder and file, the note under the settings, the reason of each choice
/// that cannot be made, and why Render cannot start.
#[derive(Debug, Clone)]
pub struct ExportDialogView {
    /// The line under the title, as in "Arrival · 00:00:28.80 → 00:00:37.40 · output 6.2 s".
    pub subtitle: Text,
    pub format: ExportFormat,
    pub resolution: ExportResolution,
    pub frame_rate: ExportFrameRate,
    /// From 0 to 1.
    pub quality: f64,
    pub quality_label: Text,
    /// The sidecar switch is on.
    pub sidecar: bool,
    pub folder: Text,
    pub file: Text,
    pub note: Text,
    /// The segments that cannot be chosen now, each with why (its tooltip).
    pub unavailable: Vec<(ExportChoice, Text)>,
    /// Why Render and Add to queue cannot go, shown in the note panel and as the buttons' tooltip; `None` when they
    /// can.
    pub render_blocked: Option<Text>,
}
