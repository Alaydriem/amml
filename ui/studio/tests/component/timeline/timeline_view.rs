use amml_studio_ui::component::timeline::{LaneScrollArea, TimelineAction, TimelineLayout, TimelineView};
use amml_ui::tokens::{SpacingToken, spacing};
use egui::{Rect, pos2, vec2};

use super::{model, track};
use crate::support::UiProbe;

fn screen() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1490.0, 400.0))
}

#[test]
fn only_lanes_scroll() {
    let mut model = model(30);
    let before = TimelineLayout::new(screen(), &model);
    model.scroll = 500.0;
    let after = TimelineLayout::new(screen(), &model);
    assert_eq!(before.ruler, after.ruler);
    assert_eq!(before.bands, after.bands);
    assert_eq!(before.realm, after.realm);
    assert_eq!(after.lanes[0].top(), before.lanes[0].top() - 500.0);
}

#[test]
fn add_track_is_the_last_lane_row() {
    let layout = TimelineLayout::new(screen(), &model(3));
    let last = layout.lanes[2];
    assert_eq!(layout.add_track.top(), last.bottom());
    assert!(layout.lanes_area.contains_rect(layout.add_track), "{:?} in {:?}", layout.add_track, layout.lanes_area);
}

#[test]
fn lane_is_the_lane_token_high() {
    let layout = TimelineLayout::new(screen(), &model(5));
    for lane in &layout.lanes {
        assert_eq!(lane.height(), <spacing::Lane as SpacingToken>::VALUE);
    }
}

#[test]
fn timeline_takes_only_its_rect() {
    let model = model(30);
    let mut probe = UiProbe::new(1490.0);
    let (used, _) = probe.frame(Vec::new(), |ui| {
        TimelineView::new(&model).show(ui);
        ui.min_rect()
    });
    assert!(used.bottom() <= 400.0, "the hidden lanes took room under the timeline: {used:?}");
}

#[test]
fn timeline_with_no_tracks() {
    let model = model(0);
    let layout = TimelineLayout::new(screen(), &model);
    assert!(layout.lanes.is_empty());
    assert_eq!(layout.add_track.top(), layout.lanes_area.top());
    let texts = UiProbe::texts(&UiProbe::new(1490.0).shapes(|ui| TimelineView::new(&model).show(ui)));
    assert!(texts.iter().any(|text| text == "Add track"), "{texts:?}");
    assert!(texts.iter().any(|text| text == "SCENES"), "{texts:?}");
}

#[test]
fn locked_lane_ignores_drags() {
    let mut model = model(0);
    model.tracks = vec![track("Camera", 6, true), track("FOV", 6, false)];
    let layout = TimelineLayout::new(screen(), &model);
    let axis = layout.axis(&model);
    let drag = |lane: usize| {
        let row = layout.lanes[lane];
        let from = pos2(axis.x_of(2.0), row.center().y);
        let mut probe = UiProbe::new(1490.0);
        probe.drag(from, from + vec2(60.0, 0.0), |ui| TimelineView::new(&model).show(ui)).concat()
    };
    let moved =
        |actions: &[TimelineAction]| actions.iter().any(|action| matches!(action, TimelineAction::MoveKey { .. }));
    assert!(moved(&drag(1)), "an open lane moves its key");
    assert!(!moved(&drag(0)), "a locked lane does not");
}

#[test]
fn key_drag_stays_a_move_when_the_host_moves_the_key() {
    let mut model = model(0);
    model.tracks = vec![track("Camera", 6, false)];
    let layout = TimelineLayout::new(screen(), &model);
    let from = pos2(layout.axis(&model).x_of(2.0), layout.lanes[0].center().y);
    let mut probe = UiProbe::new(1490.0);
    let frames = probe.drag(from, from + vec2(120.0, 0.0), |ui| {
        let actions = TimelineView::new(&model).show(ui);
        for action in &actions {
            if let TimelineAction::MoveKey { lane, key, seconds } = action {
                model.tracks[*lane].lane.keys[*key].seconds = *seconds;
            }
        }
        actions
    });
    let actions = frames.concat();
    assert!(
        !actions.iter().any(|action| matches!(action, TimelineAction::SelectKeys(_))),
        "the drag turned into a box: {actions:?}"
    );
    assert!(actions.contains(&TimelineAction::MoveKeyDone { lane: 0, key: 2 }), "{actions:?}");
    assert!((model.tracks[0].lane.keys[2].seconds - 3.2).abs() < 0.05, "{:?}", model.tracks[0].lane.keys[2]);
}

#[test]
fn no_add_track_row_when_the_flag_is_off() {
    let mut model = model(3);
    model.add_track = false;
    let layout = TimelineLayout::new(screen(), &model);
    assert_eq!(layout.add_track.height(), 0.0);
    let texts = UiProbe::texts(&UiProbe::new(1490.0).shapes(|ui| TimelineView::new(&model).show(ui)));
    assert!(!texts.iter().any(|text| text == "Add track"), "{texts:?}");
    let at = pos2(layout.lanes[2].left() + 40.0, layout.lanes[2].bottom() + 10.0);
    let actions = UiProbe::new(1490.0).click(at, |ui| TimelineView::new(&model).show(ui));
    assert!(!actions.iter().any(|action| matches!(action, TimelineAction::AddTrack)), "{actions:?}");
}

#[test]
fn the_add_track_row_takes_no_scroll_room_when_off() {
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(1490.0, 200.0));
    let on = LaneScrollArea::new(viewport, 0.0, 30, true).max_offset();
    let off = LaneScrollArea::new(viewport, 0.0, 30, false).max_offset();
    assert_eq!(on - off, LaneScrollArea::ADD_TRACK);
}

#[test]
fn a_ruler_drag_gives_scrubs_and_one_seek() {
    let model = model(0);
    let layout = TimelineLayout::new(screen(), &model);
    let axis = layout.axis(&model);
    let y = layout.ruler.center().y;
    let from = pos2(axis.x_of(1.0), y);
    let to = pos2(axis.x_of(3.0), y);
    let actions = UiProbe::new(1490.0).drag(from, to, |ui| TimelineView::new(&model).show(ui)).concat();

    let scrubs = actions.iter().filter(|action| matches!(action, TimelineAction::Scrub(_))).count();
    let seeks: Vec<&TimelineAction> = actions.iter().filter(|a| matches!(a, TimelineAction::Seek(_))).collect();
    assert!(scrubs >= 2, "{actions:?}");
    assert_eq!(seeks.len(), 1, "{actions:?}");
    assert!(matches!(seeks[0], TimelineAction::Seek(s) if (*s - 3.0).abs() < 0.05), "{seeks:?}");
}
