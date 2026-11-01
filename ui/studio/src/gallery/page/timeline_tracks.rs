use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Rect, Ui, pos2, vec2};

use super::{BarRows, TimelineSamples, ViewerSamples};
use crate::component::timeline::{BoxSelect, Lane, TimelineTrack, TimelineView, TrackHeader};
use crate::gallery::GalleryState;
use crate::tokens::selector::SceneColor;

/// Paper: Editor components, `Track headers` (`6OM-0`), `Lanes` (`6WK-0`) and `Box select, multi-select, lock`
/// (`7J7-0`): the track rows, each lane look, and the selection states.
pub struct TimelineTracksPage;

impl TimelineTracksPage {
    /// Paper `6OM-0`: the header columns are 40 apart.
    const COLUMN_GAP: f32 = 40.0;
    const COLUMN_INSET: f32 = 24.0;
    /// The Original example's height: the fixed rows and a few empty lanes.
    const ORIGINAL: f32 = 250.0;
    /// Paper `6OM-0`: the box that is being dragged across the camera and FOV lanes.
    const DRAG_BOX: egui::Vec2 = vec2(470.0, 92.0);

    pub fn track_headers(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            ViewerSamples::row(ui, "SELECTED · HOVER          HIDDEN · LOCKED", |ui| {
                let lane = <spacing::Lane as SpacingToken>::VALUE;
                TimelineSamples::place(ui, lane * 5.0, |ui, rect| {
                    let mut tracks = TimelineSamples::tracks();
                    tracks[1].header.selected = true;
                    for (index, track) in tracks.iter().enumerate() {
                        let hover = index == 2;
                        let at = pos2(rect.left() + Self::COLUMN_INSET, rect.top() + lane * index as f32);
                        let fill = if track.header.selected {
                            <color::AccentSoft as ColorToken>::VALUE
                        } else if hover {
                            <color::BgRaised as ColorToken>::VALUE
                        } else {
                            <color::BgPanel as ColorToken>::VALUE
                        };
                        TrackHeader::new(&track.header).paint(
                            ui.painter(),
                            Rect::from_min_size(at, vec2(TrackHeader::WIDTH, lane)),
                            fill,
                        );
                    }
                    let mut tracks = TimelineSamples::tracks();
                    tracks[2].header.hidden = true;
                    tracks[4].header.locked = true;
                    for (index, track) in tracks.iter().enumerate() {
                        let left = rect.left() + Self::COLUMN_INSET + TrackHeader::WIDTH + Self::COLUMN_GAP;
                        let at = pos2(left, rect.top() + lane * index as f32);
                        let place = Rect::from_min_size(at, vec2(TrackHeader::WIDTH, lane));
                        TrackHeader::new(&track.header).paint(
                            ui.painter(),
                            place,
                            <color::BgPanel as ColorToken>::VALUE,
                        );
                    }
                });
            });
        });
    }

    pub fn lanes(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            ViewerSamples::row(ui, "TIMELAPSE · CAMERA · FOV · TIME OF DAY · WEATHER", |ui| {
                Self::rows(ui, &TimelineSamples::tracks());
            });
            ViewerSamples::row(ui, "HIDDEN TRACK", |ui| {
                let mut fov = TimelineSamples::fov();
                fov.header.hidden = true;
                Self::rows(ui, &[fov]);
            });
            ViewerSamples::row(ui, "LOCKED · SCENE CAMERA DRIVES", |ui| {
                Self::rows(ui, &[Self::locked_camera()]);
            });
            ViewerSamples::row(ui, "ORIGINAL TAB", |ui| {
                let mut model = TimelineSamples::model();
                model.title = TimelineSamples::words("Original · no tracks");
                model.tracks.clear();
                model.scenes.iter_mut().for_each(|scene| scene.active = false);
                model.playhead = None;
                model.empty_hint = Some(TimelineSamples::words(
                    "Original: the replay as recorded. Pick a scene, or right-click the ruler for 'New scene here'.",
                ));
                TimelineSamples::place(ui, Self::ORIGINAL, |ui, _| TimelineView::new(&model).show(ui));
            });
        });
    }

    pub fn box_select(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            let pair = || vec![TimelineSamples::camera(), TimelineSamples::fov()];
            ViewerSamples::row(ui, "DRAGGING A BOX ACROSS CAMERA AND FOV", |ui| {
                let rows = Self::rows(ui, &pair());
                let left = rows.left() + TrackHeader::WIDTH;
                let drag = Rect::from_min_size(pos2(left + 432.0 - 222.0, rows.top() + 14.0), Self::DRAG_BOX);
                BoxSelect::new(drag).paint(ui.painter());
            });
            ViewerSamples::row(ui, "AFTER RELEASE · 5 KEYS ON 2 TRACKS", |ui| {
                let mut tracks = pair();
                for index in [1, 2, 3, 4] {
                    tracks[0].lane.keys[index].selected = true;
                }
                tracks[1].lane.keys[2].selected = true;
                Self::rows(ui, &tracks);
            });
            ViewerSamples::row(ui, "LOCKED · SCENE CAMERA DRIVES", |ui| {
                Self::rows(ui, &[Self::locked_camera()]);
            });
        });
    }

    fn locked_camera() -> TimelineTrack {
        let mut camera = TimelineSamples::camera();
        camera.header.locked = true;
        camera.lane.lock_reason = Some(TimelineSamples::words("Scene camera drives · pause to edit"));
        camera
    }

    // each track's header and lane, one under the other, with Arrival's range behind the lanes; returns their rect
    fn rows(ui: &mut Ui, tracks: &[TimelineTrack]) -> Rect {
        let lane = <spacing::Lane as SpacingToken>::VALUE;
        TimelineSamples::place(ui, lane * tracks.len() as f32, |ui, rect| {
            let axis = TimelineSamples::axis(rect.left());
            let scene = Some((28.8, 37.4, SceneColor::Lime.color()));
            for (index, track) in tracks.iter().enumerate() {
                let row =
                    Rect::from_min_size(pos2(rect.left(), rect.top() + lane * index as f32), vec2(rect.width(), lane));
                let (header, body) = TrackHeader::split(row);
                TrackHeader::new(&track.header).paint(ui.painter(), header, <color::BgPanel as ColorToken>::VALUE);
                Lane::new(&track.lane, &axis)
                    .hidden(track.header.hidden)
                    .locked(track.header.locked)
                    .scene(scene)
                    .paint(ui.painter(), body);
            }
            rect
        })
    }
}
