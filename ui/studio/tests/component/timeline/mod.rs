mod box_select;
mod ruler;
mod time_axis;
mod timeline_view;

use amml_studio_ui::component::timeline::{
    KeyKind, LaneKey, LaneView, TimelineTrack, TimelineViewModel, TrackHeaderView,
};
use amml_studio_ui::tokens::selector::TrackColor;

use crate::glyph::words;

// a camera track with a key every second for `keys` seconds
pub fn track(name: &str, keys: usize, locked: bool) -> TimelineTrack {
    TimelineTrack {
        header: TrackHeaderView {
            name: words(name),
            color: TrackColor::Camera,
            keys,
            hidden: false,
            locked,
            selected: false,
        },
        lane: LaneView {
            color: TrackColor::Camera,
            keys: (0..keys)
                .map(|index| LaneKey { seconds: index as f64, kind: KeyKind::Eased, selected: false })
                .collect(),
            spans: Vec::new(),
            curve: Vec::new(),
            lock_reason: None,
        },
    }
}

// a ten-second replay at 100 px a second with `tracks` camera tracks
pub fn model(tracks: usize) -> TimelineViewModel {
    TimelineViewModel {
        title: words("Tracks · Arrival"),
        start: 0.0,
        seconds_per_point: 0.01,
        length: 10.0,
        checkpoints: vec![5.0],
        recorded: Vec::new(),
        scenes: Vec::new(),
        realms: Vec::new(),
        tracks: (0..tracks).map(|index| track(&format!("Track {index}"), 6, false)).collect(),
        scroll: 0.0,
        playhead: None,
        bookmarks: Vec::new(),
        empty_hint: None,
        add_track: true,
    }
}
