use super::super::{LaneView, TrackHeaderView};

/// One track of the timeline: its header and its lane.
#[derive(Debug, Clone)]
pub struct TimelineTrack {
    pub header: TrackHeaderView,
    pub lane: LaneView,
}
