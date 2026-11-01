use amml_i18n::text::Text;
use egui::Pos2;

use super::{EasingFamily, EasingPreset};
use crate::tokens::selector::TrackColor;

/// What the easing popover shows.
#[derive(Debug, Clone)]
pub struct EasingPopoverView {
    /// The line under the title, as in "Camera key · applies to 3 selected keys · now: In Out Cubic".
    pub subtitle: Text,
    /// The track of the keys being eased; the curve takes its colour.
    pub track: TrackColor,
    /// The current curve, as `CurvePlotView::samples`.
    pub samples: Vec<f32>,
    /// The two handles of a custom bezier, in the unit box; `None` for a named curve.
    pub handles: Option<[Pos2; 2]>,
    /// The family the preset grid shows.
    pub family: EasingFamily,
    pub presets: Vec<EasingPreset>,
    /// The preset the keys use, when they use one.
    pub selected: Option<usize>,
    /// The user's saved presets.
    pub saved: Vec<EasingPreset>,
    pub selected_saved: Option<usize>,
}
