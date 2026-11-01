mod lane_drag;
mod menu_target;
mod timeline_action;
mod timeline_bookmark;
mod timeline_layout;
mod timeline_playhead;
mod timeline_track;
mod timeline_view_model;

pub use menu_target::MenuTarget;
pub use timeline_action::TimelineAction;
pub use timeline_bookmark::TimelineBookmark;
pub use timeline_layout::TimelineLayout;
pub use timeline_playhead::TimelinePlayhead;
pub use timeline_track::TimelineTrack;
pub use timeline_view_model::TimelineViewModel;

use lane_drag::LaneDrag;

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::glyph::Glyph;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::tokens::{ColorToken, color};
use egui::{Align2, Color32, CursorIcon, Id, Painter, Rect, Response, Sense, Stroke, Ui, UiBuilder, pos2, vec2};

use super::{
    BoxSelect, Lane, LaneKey, LaneScrollArea, RealmRow, Ruler, RulerAction, SceneBands, SceneBandsAction, TimeAxis,
    TrackHeader, TrackHeaderAction,
};
use crate::glyph::{BookmarkFlag, Playhead};
use crate::id::StudioUiId;
use crate::message::StudioUiMessage;
use crate::tokens::color as studio;

/// Paper: `Editor / Timeline · assembled` (`54P-0`). The ruler, scene bands and realm row on top, the lanes in their
/// scroll area under them with "Add track" last, and the scroll bar row at the bottom; the active scene's edges, the
/// bookmarks and the playhead are drawn over them. It fills the space it is given.
pub struct TimelineView<'a> {
    model: &'a TimelineViewModel,
}

impl<'a> TimelineView<'a> {
    // Paper: the playhead and bookmark flags start 4 below the ruler's top
    const MARKS_TOP: f32 = 4.0;
    // Paper: the active scene's edges are its colour at 75 %
    const EDGE_ALPHA: u8 = 0xBF;
    // Paper: the add track row has its icon 15 in and its label 9 after it
    const ADD_X: f32 = 15.0;
    const ADD_GAP: f32 = 9.0;
    // Paper: the scroll bar track is 10 high, 8 in, 5 down; the thumb at least 24 wide; scene marks 2 × 8 at 70 %
    const BAR_INSET: f32 = 8.0;
    const BAR_TOP: f32 = 5.0;
    const BAR_HEIGHT: f32 = 10.0;
    const THUMB_MIN: f32 = 24.0;
    const THUMB_FILL_ALPHA: u8 = 0x59;
    const THUMB_BORDER_ALPHA: u8 = 0x66;
    const MARK_WIDTH: f32 = 2.0;
    const MARK_HEIGHT: f32 = 8.0;
    const MARK_ALPHA: u8 = 0xB3;

    pub fn new(model: &'a TimelineViewModel) -> Self {
        Self { model }
    }

    pub fn show(&self, ui: &mut Ui) -> Vec<TimelineAction> {
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        let layout = TimelineLayout::new(rect, self.model);
        let axis = layout.axis(self.model);
        let mut actions = Vec::new();
        let ruler = Ruler::new(&axis, self.model.length)
            .title(&self.model.title)
            .checkpoints(&self.model.checkpoints)
            .recorded(&self.model.recorded);
        match ruler.show_at(ui, layout.ruler) {
            Some(RulerAction::Scrub(seconds)) => actions.push(TimelineAction::Scrub(seconds)),
            Some(RulerAction::Seek(seconds)) => actions.push(TimelineAction::Seek(seconds)),
            Some(RulerAction::OpenMenu(seconds, at)) => {
                actions.push(TimelineAction::OpenMenu(MenuTarget::Ruler(seconds), at));
            }
            None => {}
        }
        match SceneBands::new(&axis, &self.model.scenes).show_at(ui, layout.bands) {
            Some(SceneBandsAction::Select(index)) => actions.push(TimelineAction::SelectScene(index)),
            Some(SceneBandsAction::OpenMenu(index, at)) => {
                actions.push(TimelineAction::OpenMenu(MenuTarget::Scene(index), at));
            }
            Some(SceneBandsAction::Add) => actions.push(TimelineAction::AddScene),
            None => {}
        }
        RealmRow::new(&axis, &self.model.realms).paint(ui.painter(), layout.realm);
        let lanes = ui.scope_builder(UiBuilder::new().max_rect(layout.lanes_area), |ui| {
            ui.set_clip_rect(layout.lanes_area.intersect(ui.clip_rect()));
            self.lanes(ui, &layout, &axis)
        });
        actions.extend(lanes.inner);
        if let Some(start) = self.scroll_bar(ui, layout.scroll_bar, &axis) {
            actions.push(TimelineAction::Pan(start));
        }
        self.overlay(ui.painter(), &layout, &axis);
        // `allocate_rect` moved the cursor to each part; the page goes on under the timeline
        ui.advance_cursor_after_rect(rect);
        actions
    }

    // the track rows, the add track row and every lane interaction
    fn lanes(&self, ui: &mut Ui, layout: &TimelineLayout, axis: &TimeAxis) -> Vec<TimelineAction> {
        let mut actions = Vec::new();
        let area =
            LaneScrollArea::new(layout.lanes_area, self.model.scroll, self.model.tracks.len(), self.model.add_track);
        let (_, body) = TrackHeader::split(layout.lanes_area);
        ui.painter().rect_filled(body, 0, <color::BgApp as ColorToken>::VALUE);
        let input = ui.allocate_rect(body, Sense::click_and_drag());
        let scene = self.active_scene();
        for (index, (track, row)) in self.model.tracks.iter().zip(&layout.lanes).enumerate() {
            if !row.intersects(layout.lanes_area) {
                continue;
            }
            let (header, lane) = TrackHeader::split(*row);
            match TrackHeader::new(&track.header).show_at(ui, header) {
                Some(TrackHeaderAction::Select) => actions.push(TimelineAction::SelectTrack(index)),
                Some(TrackHeaderAction::ToggleHidden) => actions.push(TimelineAction::ToggleHidden(index)),
                Some(TrackHeaderAction::ToggleLocked) => actions.push(TimelineAction::ToggleLocked(index)),
                None => {}
            }
            self.lane(index, axis).scene(scene).paint(ui.painter(), lane);
        }
        if self.model.add_track && self.add_track(ui, area.add_track()) {
            actions.push(TimelineAction::AddTrack);
        }
        if self.model.tracks.is_empty()
            && let Some(hint) = &self.model.empty_hint
        {
            let style = TextStyle::new(TypeSize::Note, FontFace::Regular);
            let faint = <color::TextFaint as ColorToken>::VALUE;
            let room = (body.width() - TrackHeader::WIDTH).max(0.0);
            Draw::text(ui.painter(), body.center(), Align2::CENTER_CENTER, hint, style, faint, TextFit::Truncate(room));
        }
        actions.extend(self.lane_input(ui, &input, layout, axis));
        if let Some(offset) = area.scrolled(ui) {
            actions.push(TimelineAction::Scroll(offset));
        }
        actions
    }

    fn lane(&self, index: usize, axis: &'a TimeAxis) -> Lane<'_> {
        let track = &self.model.tracks[index];
        Lane::new(&track.lane, axis).hidden(track.header.hidden).locked(track.header.locked)
    }

    // the lane a point is on, and the key under it
    fn hit(&self, layout: &TimelineLayout, axis: &TimeAxis, at: egui::Pos2) -> Option<(usize, Option<usize>)> {
        let index = layout.lanes.iter().position(|row| row.contains(at))?;
        let (_, lane) = TrackHeader::split(layout.lanes[index]);
        Some((index, self.lane(index, axis).key_at(lane, at)))
    }

    // clicks, right-clicks and drags on the lane bodies: select a key, open a menu, move a key, or drag a box
    fn lane_input(&self, ui: &Ui, input: &Response, layout: &TimelineLayout, axis: &TimeAxis) -> Vec<TimelineAction> {
        let mut actions = Vec::new();
        let pointer = input.interact_pointer_pos();
        if let Some(at) = pointer.filter(|_| input.secondary_clicked()) {
            match self.hit(layout, axis, at) {
                Some((lane, Some(key))) => actions.push(TimelineAction::OpenMenu(MenuTarget::Key { lane, key }, at)),
                Some((lane, None)) => actions.push(TimelineAction::OpenMenu(MenuTarget::Lane(lane), at)),
                None => {}
            }
        } else if let Some(at) = pointer.filter(|_| input.clicked())
            && let Some((lane, Some(key))) = self.hit(layout, axis, at)
        {
            actions.push(TimelineAction::SelectKey { lane, key });
        }
        let origin = ui.input(|state| state.pointer.press_origin());
        let id = Id::unique(StudioUiId::TimelineDrag);
        if let (true, Some(origin)) = (input.drag_started(), origin) {
            let drag = match self.hit(layout, axis, origin) {
                Some((lane, Some(_))) if self.model.tracks[lane].header.locked => LaneDrag::Locked,
                Some((lane, Some(key))) => LaneDrag::Move { lane, key },
                _ => LaneDrag::Box,
            };
            ui.ctx().data_mut(|data| data.insert_temp(id, drag));
        }
        let drag = ui.ctx().data(|data| data.get_temp::<LaneDrag>(id));
        if input.drag_stopped() {
            if let Some(LaneDrag::Move { lane, key }) = drag {
                actions.push(TimelineAction::MoveKeyDone { lane, key });
            }
            ui.ctx().data_mut(|data| data.remove::<LaneDrag>(id));
        }
        if let (true, Some(origin), Some(at), Some(drag)) = (input.dragged(), origin, pointer, drag) {
            match drag {
                LaneDrag::Move { lane, key } => {
                    actions.push(TimelineAction::MoveKey { lane, key, seconds: axis.seconds_at(at.x).max(0.0) });
                }
                LaneDrag::Locked => {}
                LaneDrag::Box => {
                    let drag = BoxSelect::new(Rect::from_two_pos(origin, at).intersect(layout.lanes_area));
                    drag.paint(ui.painter());
                    let lanes: Vec<(Rect, &[LaneKey])> = self
                        .model
                        .tracks
                        .iter()
                        .zip(&layout.lanes)
                        .map(|(track, row)| (*row, track.lane.keys.as_slice()))
                        .collect();
                    actions.push(TimelineAction::SelectKeys(drag.pick(axis, &lanes)));
                }
            }
        }
        actions
    }

    // the "Add track" row; true when clicked
    fn add_track(&self, ui: &mut Ui, row: Rect) -> bool {
        let (header, _) = TrackHeader::split(row);
        let shown = TrackHeader::visible(ui, header);
        let response = ui.allocate_rect(shown, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        let painter = ui.painter();
        let fill = if response.hovered() {
            <color::BgRaised as ColorToken>::VALUE
        } else {
            <color::BgPanel as ColorToken>::VALUE
        };
        painter.rect_filled(header, 0, fill);
        painter.vline(header.right() - 0.5, header.y_range(), Stroke::new(1.0, <color::Line as ColorToken>::VALUE));
        let dim = <color::TextDim as ColorToken>::VALUE;
        let icon = IconSize::Small.points();
        let place =
            Rect::from_min_size(pos2(header.left() + Self::ADD_X, header.center().y - icon / 2.0), vec2(icon, icon));
        Icon::Add.paint(painter, place, IconSize::Small, dim);
        let label = Text::message(StudioUiMessage::AddTrack);
        let style = TextStyle::new(TypeSize::Help, FontFace::Regular);
        let at = pos2(place.right() + Self::ADD_GAP, header.center().y);
        Draw::text(painter, at, Align2::LEFT_CENTER, &label, style, dim, TextFit::Line);
        response.clicked()
    }

    // the scroll bar row: the view's place in the replay, with a mark at each scene start; a drag pans
    fn scroll_bar(&self, ui: &mut Ui, rect: Rect, axis: &TimeAxis) -> Option<f64> {
        let (header, area) = TrackHeader::split(rect);
        let painter = ui.painter();
        painter.rect_filled(header, 0, <color::BgPanel as ColorToken>::VALUE);
        painter.vline(header.right() - 0.5, header.y_range(), Stroke::new(1.0, <color::Line as ColorToken>::VALUE));
        painter.rect_filled(area, 0, <color::BgPanel as ColorToken>::VALUE);
        let track = Rect::from_min_size(
            pos2(area.left() + Self::BAR_INSET, area.top() + Self::BAR_TOP),
            vec2((area.width() - Self::BAR_INSET * 2.0).max(0.0), Self::BAR_HEIGHT),
        );
        Draw::fill(painter, track, Corner::Small, <color::BgField as ColorToken>::VALUE);
        let length = self.model.length.max(f64::EPSILON);
        let x_of = |seconds: f64| track.left() + (seconds / length) as f32 * track.width();
        for scene in &self.model.scenes {
            let mark = Rect::from_center_size(
                pos2(x_of(scene.start), track.center().y),
                vec2(Self::MARK_WIDTH, Self::MARK_HEIGHT),
            );
            painter.rect_filled(mark, 0, scene.color.color().gamma_multiply_u8(Self::MARK_ALPHA));
        }
        let visible = f64::from(area.width()) * axis.seconds_per_point;
        let width =
            ((visible / length) as f32 * track.width()).clamp(Self::THUMB_MIN.min(track.width()), track.width());
        let left = x_of(self.model.start).clamp(track.left(), track.right() - width);
        let thumb = Rect::from_min_size(pos2(left, track.top()), vec2(width, track.height()));
        let response = ui.allocate_rect(thumb, Sense::drag()).on_hover_cursor(CursorIcon::Grab);
        let painter = ui.painter();
        Draw::fill(
            painter,
            thumb,
            Corner::Small,
            <color::TextDim as ColorToken>::VALUE.gamma_multiply_u8(Self::THUMB_FILL_ALPHA),
        );
        Draw::stroke(
            painter,
            thumb,
            Corner::Small,
            <color::Text as ColorToken>::VALUE.gamma_multiply_u8(Self::THUMB_BORDER_ALPHA),
        );
        let delta = response.drag_delta().x;
        (response.dragged() && delta != 0.0 && track.width() > 0.0).then(|| {
            (self.model.start + f64::from(delta / track.width()) * length).clamp(0.0, (length - visible).max(0.0))
        })
    }

    // the active scene's edges, the bookmarks and the playhead, over the time area
    fn overlay(&self, painter: &Painter, layout: &TimelineLayout, axis: &TimeAxis) {
        let (_, time) = TrackHeader::split(Rect::from_min_max(
            layout.ruler.min,
            pos2(layout.ruler.right(), layout.lanes_area.bottom()),
        ));
        let painter = painter.with_clip_rect(time.intersect(painter.clip_rect()));
        if let Some((start, end, hue)) = self.active_scene() {
            let edge = Stroke::new(1.0, hue.gamma_multiply_u8(Self::EDGE_ALPHA));
            for seconds in [start, end] {
                let x = axis.x_of(seconds).round() + 0.5;
                painter.vline(x, layout.bands.top()..=time.bottom(), edge);
            }
        }
        let top = layout.ruler.top() + Self::MARKS_TOP;
        for bookmark in &self.model.bookmarks {
            let x = axis.x_of(bookmark.seconds);
            let flag = Rect::from_min_max(pos2(x, top), pos2(x + BookmarkFlag::WIDTH, time.bottom()));
            BookmarkFlag { label: bookmark.label.clone() }.paint(&painter, flag, <studio::Marker as ColorToken>::VALUE);
        }
        if let Some(playhead) = &self.model.playhead {
            let x = axis.x_of(playhead.seconds);
            let place = Rect::from_min_max(
                pos2(x - Playhead::HEAD_WIDTH / 2.0, top),
                pos2(x + Playhead::HEAD_WIDTH / 2.0, time.bottom()),
            );
            let hue: Color32 = if playhead.scene_drives {
                <studio::SceneCamera as ColorToken>::VALUE
            } else {
                <color::Playhead as ColorToken>::VALUE
            };
            Playhead { label: playhead.label.clone() }.paint(&painter, place, hue);
        }
    }

    // the active scene's start, end and colour
    fn active_scene(&self) -> Option<(f64, f64, Color32)> {
        self.model.scenes.iter().find(|scene| scene.active).map(|scene| (scene.start, scene.end, scene.color.color()))
    }
}
