use amml_i18n::text::{Text, UserText};
use egui::{Rect, Ui, UiBuilder};

use crate::component::timeline::{
    KeyKind, LaneKey, LaneSpan, LaneView, RealmSegment, SceneBand, SpanLook, TimeAxis, TimelineBookmark,
    TimelinePlayhead, TimelineTrack, TimelineViewModel, TrackHeader, TrackHeaderView,
};
use crate::tokens::selector::{RealmColor, SceneColor, TrackColor};

/// The sample replay the timeline pages share: Paper's Arrival scene at 101 px a second (`54P-0`, `6WK-0`).
pub struct TimelineSamples;

impl TimelineSamples {
    /// Paper: the timeline is 1490 wide.
    pub const WIDTH: f32 = 1490.0;
    /// Paper: 101 px a second, with Arrival's start (28.80 s) 70 px into the lanes.
    pub const SECONDS_PER_POINT: f64 = 1.0 / 101.0;
    pub const START: f64 = 28.8 - 70.0 / 101.0;

    pub fn words(text: &str) -> Text {
        Text::user(UserText::from_model(text.to_owned()))
    }

    pub fn axis(left: f32) -> TimeAxis {
        TimeAxis { start: Self::START, seconds_per_point: Self::SECONDS_PER_POINT, left: left + TrackHeader::WIDTH }
    }

    /// Shows `body` in a rect `height` high, the page's width, at the cursor.
    pub fn place<R>(ui: &mut Ui, height: f32, body: impl FnOnce(&mut Ui, Rect) -> R) -> R {
        let rect = Rect::from_min_size(ui.cursor().min, egui::vec2(Self::WIDTH, height));
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // the scope then takes the whole rect, whether or not `body` allocates
            ui.set_min_size(rect.size());
            body(ui, rect)
        })
        .inner
    }

    pub fn model() -> TimelineViewModel {
        TimelineViewModel {
            title: Self::words("Tracks · Arrival"),
            start: Self::START,
            seconds_per_point: Self::SECONDS_PER_POINT,
            length: 134.0,
            checkpoints: vec![30.0, 35.0, 40.0],
            recorded: Vec::new(),
            scenes: Self::scenes(0, None),
            realms: vec![RealmSegment {
                start: 0.0,
                end: 134.0,
                realm: RealmColor::Overworld,
                name: Self::words("Overworld"),
            }],
            tracks: Self::tracks(),
            scroll: 0.0,
            playhead: Some(TimelinePlayhead { seconds: 32.95, label: Self::words("32.95"), scene_drives: false }),
            bookmarks: vec![TimelineBookmark { seconds: Self::START + 741.0 / 101.0, label: Self::words("portal") }],
            empty_hint: None,
            add_track: true,
        }
    }

    /// The scenes of `6D4-0`: Fortress clipped at the left, Arrival, Portal run, and Multi-angle B on row 2. `active`
    /// is the index of the active scene; `arrival_end` moves Arrival's out point.
    pub fn scenes(active: usize, arrival_end: Option<(f64, &str)>) -> Vec<SceneBand> {
        let (end, range) = arrival_end.unwrap_or((37.4, "00:28.80 – 00:37.40"));
        let band = |name: &str, range: &str, start: f64, end: f64, color: SceneColor, row: usize| SceneBand {
            name: Self::words(name),
            range: Self::words(range),
            start,
            end,
            color,
            row,
            active: false,
        };
        let mut scenes = vec![
            band("Arrival", range, 28.8, end, SceneColor::Lime, 0),
            band("Portal run", "00:37.90 – 00:52.00", 37.9, 52.0, SceneColor::Pink, 0),
            band("Fortress", "00:26.10 – 00:28.30", 26.1, 28.3, SceneColor::Orchid, 1),
            band("Multi-angle B", "00:31.20 – 00:36.60", 31.2, 36.6, SceneColor::Periwinkle, 1),
        ];
        scenes[active].active = true;
        scenes
    }

    pub fn tracks() -> Vec<TimelineTrack> {
        vec![Self::timelapse(), Self::camera(), Self::fov(), Self::time_of_day(), Self::weather()]
    }

    fn track(name: &str, color: TrackColor, keys: Vec<LaneKey>, spans: Vec<LaneSpan>) -> TimelineTrack {
        TimelineTrack {
            header: TrackHeaderView {
                name: Self::words(name),
                color,
                keys: keys.len(),
                hidden: false,
                locked: false,
                selected: false,
            },
            lane: LaneView { color, keys, spans, curve: Vec::new(), lock_reason: None },
        }
    }

    fn keys(keys: &[(f64, KeyKind)]) -> Vec<LaneKey> {
        keys.iter().map(|(seconds, kind)| LaneKey { seconds: *seconds, kind: *kind, selected: false }).collect()
    }

    fn span(start: f64, end: f64, label: &str, look: SpanLook) -> LaneSpan {
        LaneSpan { start, end, label: Some(Self::words(label)), look }
    }

    pub fn timelapse() -> TimelineTrack {
        let keys = Self::keys(&[(28.8, KeyKind::Eased), (32.94, KeyKind::Eased), (37.37, KeyKind::Eased)]);
        let spans = vec![
            Self::span(28.8, 32.94, "1.00× · plays in 4.2 s", SpanLook::Fill { strength: 0.0 }),
            Self::span(32.94, 37.37, "2.20× · plays in 2.0 s", SpanLook::Fill { strength: 1.0 }),
        ];
        Self::track("Timelapse", TrackColor::Timelapse, keys, spans)
    }

    pub fn camera() -> TimelineTrack {
        let times = [28.8, 30.33, 32.94, 35.37, 37.0, 37.37];
        let keys = Self::keys(&[
            (times[0], KeyKind::Auto),
            (times[1], KeyKind::Eased),
            (times[2], KeyKind::Eased),
            (times[3], KeyKind::Custom),
            (times[4], KeyKind::Eased),
            (times[5], KeyKind::Hold),
        ]);
        let eases = ["Smooth", "Smooth", "In Out Cubic", "Custom", "Linear"];
        let spans = eases
            .iter()
            .enumerate()
            .map(|(index, ease)| Self::span(times[index], times[index + 1], ease, SpanLook::Ease { hold: false }))
            .collect();
        Self::track("Camera", TrackColor::Camera, keys, spans)
    }

    pub fn fov() -> TimelineTrack {
        let keys = Self::keys(&[
            (28.8, KeyKind::Eased),
            (31.4, KeyKind::Eased),
            (34.7, KeyKind::Eased),
            (37.12, KeyKind::Hold),
        ]);
        let mut track = Self::track("FOV", TrackColor::Fov, keys, Vec::new());
        track.lane.curve = vec![(28.8, 0.35), (31.4, 0.8), (34.7, 0.2), (37.12, 0.55), (40.0, 0.55)];
        track
    }

    pub fn time_of_day() -> TimelineTrack {
        let keys = Self::keys(&[(28.8, KeyKind::Hold), (35.78, KeyKind::Hold)]);
        let spans = vec![
            Self::span(28.8, 35.78, "12:00", SpanLook::Fill { strength: 0.6 }),
            Self::span(35.78, 37.4, "18:30", SpanLook::Fill { strength: 0.9 }),
        ];
        Self::track("Time of day", TrackColor::Time, keys, spans)
    }

    pub fn weather() -> TimelineTrack {
        let keys = Self::keys(&[(28.8, KeyKind::Hold), (33.83, KeyKind::Hold)]);
        let spans = vec![
            Self::span(28.8, 33.83, "Clear", SpanLook::Clip { strength: 0.0 }),
            Self::span(33.83, 37.37, "Rain", SpanLook::Clip { strength: 1.0 }),
        ];
        Self::track("Weather", TrackColor::Weather, keys, spans)
    }
}
