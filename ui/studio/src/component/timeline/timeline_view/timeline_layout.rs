use egui::{Rect, pos2};

use super::super::{LaneScrollArea, RealmRow, Ruler, SceneBands, TimeAxis, TrackHeader};
use super::TimelineViewModel;

/// Where each part of the timeline sits in its rect: the fixed rows on top, the lane scroll area, and the scroll bar
/// row at the bottom. Only the lane rows move with the scroll.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineLayout {
    pub ruler: Rect,
    pub bands: Rect,
    pub realm: Rect,
    /// The viewport of the lane scroll area, header column included.
    pub lanes_area: Rect,
    /// Each track's row, header and lane, after the scroll.
    pub lanes: Vec<Rect>,
    pub add_track: Rect,
    pub scroll_bar: Rect,
}

impl TimelineLayout {
    /// Paper: the scroll bar row is 20 high.
    pub const SCROLL_BAR: f32 = 20.0;

    pub fn new(rect: Rect, model: &TimelineViewModel) -> Self {
        let row = |top: f32, height: f32| Rect::from_min_max(pos2(rect.left(), top), pos2(rect.right(), top + height));
        let ruler = row(rect.top(), Ruler::HEIGHT);
        let bands = row(ruler.bottom(), SceneBands::HEIGHT);
        let realm = row(bands.bottom(), RealmRow::HEIGHT);
        let scroll_bar = row((rect.bottom() - Self::SCROLL_BAR).max(realm.bottom()), Self::SCROLL_BAR);
        let lanes_area = Rect::from_min_max(pos2(rect.left(), realm.bottom()), pos2(rect.right(), scroll_bar.top()));
        let area = LaneScrollArea::new(lanes_area, model.scroll, model.tracks.len(), model.add_track);
        Self {
            ruler,
            bands,
            realm,
            lanes_area,
            lanes: (0..model.tracks.len()).map(|index| area.row(index)).collect(),
            add_track: area.add_track(),
            scroll_bar,
        }
    }

    /// The time axis of this layout: the model's zoom, starting at the lanes' left edge.
    pub fn axis(&self, model: &TimelineViewModel) -> TimeAxis {
        TimeAxis {
            start: model.start,
            seconds_per_point: model.seconds_per_point,
            left: self.ruler.left() + TrackHeader::WIDTH,
        }
    }
}
