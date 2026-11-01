use amml_ui::glyph::Glyph;
use amml_ui::tokens::{ColorToken, color};
use egui::{Rect, Ui, pos2};

use super::{BarRows, TimelineSamples, ViewerSamples};
use crate::component::timeline::{RealmRow, RealmSegment, Ruler, SceneBands, TimeAxis, TrackHeader};
use crate::gallery::GalleryState;
use crate::glyph::{BookmarkFlag, Playhead};
use crate::tokens::color as studio;
use crate::tokens::selector::RealmColor;

/// Paper: Editor components, `Ruler` (`5Y5-0`), `Scene bands` (`6D4-0`) and `Realm row` (`6IW-0`): the fixed rows on
/// top of the timeline, each in the states Paper draws.
pub struct TimelineRowsPage;

impl TimelineRowsPage {
    // Paper `6IW-0` zoomed out: the 134 s replay across the 1268 px lanes, about 0.1 s a point
    const ZOOMED_OUT: f64 = 0.1;
    // Paper `5Y5-0`, state "Recorded segment"
    const RECORDED: [(f64, f64); 1] = [(29.47, 37.0)];

    pub fn ruler(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            let states = [
                ("PAUSED", false, &[][..]),
                ("PLAYING · SCENE CAMERA DRIVES, BOOKMARK HOVERED", true, &[][..]),
                ("RECORDED SEGMENT · 00:29.47 – 00:37.00", false, &Self::RECORDED[..]),
            ];
            for (caption, playing, recorded) in states {
                ViewerSamples::row(ui, caption, |ui| {
                    TimelineSamples::place(ui, Ruler::HEIGHT, |ui, rect| {
                        let axis = TimelineSamples::axis(rect.left());
                        let title = TimelineSamples::words("Tracks · Arrival");
                        Ruler::new(&axis, 134.0)
                            .title(&title)
                            .checkpoints(&[30.0, 35.0, 40.0])
                            .recorded(recorded)
                            .show_at(ui, rect);
                        // Paper: the recorded segment's state has the playhead at the segment's start
                        let start = recorded.first().map(|&(start, _)| (start, format!("{start:.2}")));
                        let playhead = start.as_ref().map(|(start, label)| (*start, label.as_str()));
                        Self::marks(ui, rect, &axis, playing, playhead);
                    });
                });
            }
        });
    }

    // the bookmark and the playhead on a ruler
    fn marks(ui: &Ui, rect: Rect, axis: &TimeAxis, playing: bool, playhead: Option<(f64, &str)>) {
        let top = rect.top() + 4.0;
        let x = axis.x_of(TimelineSamples::START + 741.0 / 101.0);
        let flag = Rect::from_min_max(pos2(x, top), pos2(x + BookmarkFlag::WIDTH, rect.bottom()));
        let flag_hue =
            if playing { <color::OnAccent as ColorToken>::VALUE } else { <studio::Marker as ColorToken>::VALUE };
        BookmarkFlag { label: TimelineSamples::words("portal") }.paint(ui.painter(), flag, flag_hue);
        let (seconds, label, hue) = if let Some((seconds, label)) = playhead {
            (seconds, label, <color::Playhead as ColorToken>::VALUE)
        } else if playing {
            (31.12, "31.12", <studio::SceneCamera as ColorToken>::VALUE)
        } else {
            (32.95, "32.95", <color::Playhead as ColorToken>::VALUE)
        };
        let x = axis.x_of(seconds);
        let head = Rect::from_min_max(
            pos2(x - Playhead::HEAD_WIDTH / 2.0, top),
            pos2(x + Playhead::HEAD_WIDTH / 2.0, rect.bottom()),
        );
        Playhead { label: TimelineSamples::words(label) }.paint(ui.painter(), head, hue);
    }

    pub fn scene_bands(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            let states = [
                ("ARRIVAL ACTIVE · FORTRESS SPILLS OVER", 0, None),
                ("MULTI-ANGLE B ACTIVE ON ROW 2", 3, None),
                ("ARRIVAL OUT POINT AT 00:35.00", 0, Some((35.0, "00:28.80 – 00:35.00"))),
            ];
            for (caption, active, end) in states {
                ViewerSamples::row(ui, caption, |ui| {
                    let scenes = TimelineSamples::scenes(active, end);
                    TimelineSamples::place(ui, SceneBands::HEIGHT, |ui, rect| {
                        let axis = TimelineSamples::axis(rect.left());
                        SceneBands::new(&axis, &scenes).show_at(ui, rect);
                    });
                });
            }
        });
    }

    pub fn realm_row(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            let segment = |start: f64, end: f64, realm: RealmColor, name: &str| RealmSegment {
                start,
                end,
                realm,
                name: TimelineSamples::words(name),
            };
            // Paper: zoomed in, Nether starts at the lanes' right edge
            let edge = TimelineSamples::START + 1251.0 / 101.0;
            let zoomed_in = vec![
                segment(0.0, edge, RealmColor::Overworld, "Overworld"),
                segment(edge, edge + 4.0, RealmColor::Nether, "Nether"),
            ];
            let zoomed_out = vec![
                segment(0.0, 30.0, RealmColor::Overworld, "Overworld"),
                segment(30.0, 82.0, RealmColor::Nether, "Nether"),
                segment(82.0, 106.0, RealmColor::End, "End"),
                segment(106.0, 134.0, RealmColor::Overworld, "Overworld"),
            ];
            ViewerSamples::row(ui, "101 PX A SECOND · NETHER AT THE RIGHT EDGE", |ui| {
                TimelineSamples::place(ui, RealmRow::HEIGHT, |ui, rect| {
                    RealmRow::new(&TimelineSamples::axis(rect.left()), &zoomed_in).paint(ui.painter(), rect);
                });
            });
            ViewerSamples::row(ui, "ZOOMED OUT", |ui| {
                TimelineSamples::place(ui, RealmRow::HEIGHT, |ui, rect| {
                    let axis = TimeAxis {
                        start: 0.0,
                        seconds_per_point: Self::ZOOMED_OUT,
                        left: rect.left() + TrackHeader::WIDTH,
                    };
                    RealmRow::new(&axis, &zoomed_out).paint(ui.painter(), rect);
                });
            });
        });
    }
}
