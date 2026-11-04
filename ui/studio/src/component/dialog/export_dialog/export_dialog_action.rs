use super::{ExportFormat, ExportFrameRate, ExportH265Profile, ExportProResProfile, ExportResolution};

/// What the export dialog asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportDialogAction {
    SetFormat(ExportFormat),
    SetResolution(ExportResolution),
    SetFrameRate(ExportFrameRate),
    SetQuality(f64),
    SetH265Profile(ExportH265Profile),
    SetProResProfile(ExportProResProfile),
    SetSidecar(bool),
    /// A click on the output field: pick a folder.
    ChooseOutput,
    /// Render the scene now.
    RenderNow,
    /// Add the scene to the render queue.
    AddToQueue,
    /// Show the render queue.
    OpenQueue,
    /// The close mark, Escape or a click outside.
    Cancel,
}
