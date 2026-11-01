use amml_i18n::text::Text;

use super::super::{RealmSegment, SceneBand};
use super::{TimelineBookmark, TimelinePlayhead, TimelineTrack};

/// Everything the timeline shows. The host keeps the zoom (`start`, `seconds_per_point`) and the lane `scroll`, and
/// changes them on `Pan` and `Scroll`.
#[derive(Debug, Clone)]
pub struct TimelineViewModel {
    /// The ruler's header cell, as in "Tracks · Arrival".
    pub title: Text,
    /// The time at the left edge of the lanes.
    pub start: f64,
    pub seconds_per_point: f64,
    /// The replay's length in seconds.
    pub length: f64,
    pub checkpoints: Vec<f64>,
    /// The parts the user recorded, as `(start, end)` seconds with start before end.
    pub recorded: Vec<(f64, f64)>,
    pub scenes: Vec<SceneBand>,
    pub realms: Vec<RealmSegment>,
    pub tracks: Vec<TimelineTrack>,
    pub scroll: f32,
    pub playhead: Option<TimelinePlayhead>,
    pub bookmarks: Vec<TimelineBookmark>,
    /// The hint centred over the empty lanes when there are no tracks (the Original replay).
    pub empty_hint: Option<Text>,
    /// Draws the Add track row under the last track; off, the row takes no height.
    pub add_track: bool,
}
