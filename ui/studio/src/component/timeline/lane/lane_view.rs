use amml_i18n::text::Text;

use super::{LaneKey, LaneSpan};
use crate::tokens::selector::TrackColor;

/// What a lane shows: its keys, the spans between them, and for a value track (FOV) the curve through its values.
#[derive(Debug, Clone)]
pub struct LaneView {
    pub color: TrackColor,
    pub keys: Vec<LaneKey>,
    pub spans: Vec<LaneSpan>,
    /// `(seconds, value)` points, each value from 0 (the lowest in view) to 1 (the highest). Empty for no curve.
    pub curve: Vec<(f64, f32)>,
    /// Why the lane cannot be edited, shown in the lock pill while the track is locked.
    pub lock_reason: Option<Text>,
}
