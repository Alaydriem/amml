use amml_studio_ui::component::timeline::{Ruler, RulerAction, TimeAxis, TrackHeader};
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};

use crate::glyph::words;
use crate::support::UiProbe;

// the painted time labels of a ruler 1490 wide
fn label_rects(length: f64, seconds_per_point: f64) -> Vec<egui::Rect> {
    let axis = TimeAxis { start: 0.0, seconds_per_point, left: TrackHeader::WIDTH };
    let title = words("Tracks");
    let shapes = UiProbe::new(1490.0).shapes(|ui| Ruler::new(&axis, length).title(&title).show(ui));
    let mut rects: Vec<egui::Rect> = shapes
        .iter()
        .filter_map(|shape| match shape {
            egui::Shape::Text(text) if text.galley.text() != "Tracks" => {
                Some(text.galley.rect.translate(text.pos.to_vec2()))
            }
            _ => None,
        })
        .collect();
    rects.sort_by(|a, b| a.left().total_cmp(&b.left()));
    rects
}

#[test]
fn ruler_labels_never_overlap() {
    for length in [1.0, 600.0, 10_800.0] {
        // from one frame (1/20 s) across the lane width to the whole replay in 1280 px
        for seconds_per_point in [0.05 / 1268.0, 0.002, 0.01, 0.1, 1.0, length / 1280.0] {
            let rects = label_rects(length, seconds_per_point);
            assert!(!rects.is_empty(), "no labels for {length} s at {seconds_per_point}");
            for pair in rects.windows(2) {
                assert!(
                    pair[0].right() <= pair[1].left(),
                    "labels overlap for {length} s at {seconds_per_point}: {pair:?}"
                );
            }
        }
    }
}

// a 1490 wide ruler at 100 px a second whose recorded ranges are `ranges`; returns the painted shapes
fn recorded_shapes(ranges: &[(f64, f64)]) -> Vec<egui::Shape> {
    let axis = TimeAxis { start: 0.0, seconds_per_point: 0.01, left: TrackHeader::WIDTH };
    UiProbe::new(1490.0).shapes(|ui| Ruler::new(&axis, 10.0).recorded(ranges).show(ui))
}

#[test]
fn recorded_segments_are_painted_behind_the_ticks() {
    let shapes = recorded_shapes(&[(2.0, 4.0)]);
    let soft = <color::AccentSoft as ColorToken>::VALUE;
    let edge = <color::Accent as ColorToken>::VALUE;
    let left = TrackHeader::WIDTH + 200.0;
    let right = TrackHeader::WIDTH + 400.0;
    let rects = UiProbe::rects(&shapes);
    let fill = rects.iter().find(|r| r.fill == soft).expect("no accent-soft fill");
    assert_eq!((fill.rect.left(), fill.rect.right()), (left, right));
    assert_eq!(fill.rect.height(), Ruler::HEIGHT);
    let bar = rects.iter().find(|r| r.fill == edge).expect("no accent edge");
    assert_eq!((bar.rect.left(), bar.rect.right()), (left, right));
    assert_eq!(bar.rect.height(), <spacing::Indicator as SpacingToken>::VALUE);
    assert_eq!(bar.rect.bottom(), Ruler::HEIGHT);
    let order =
        |wanted: egui::Color32| shapes.iter().position(|s| matches!(s, egui::Shape::Rect(r) if r.fill == wanted));
    // a tick is a vertical line inside the time area (the dividers are horizontal, or on the header's edge)
    let first_tick = shapes
        .iter()
        .position(|s| {
            matches!(s, egui::Shape::LineSegment { points, .. }
                if points[0].x == points[1].x && points[0].x > TrackHeader::WIDTH)
        })
        .expect("no ticks");
    assert!(order(soft).expect("fill") < first_tick);
    assert!(order(edge).expect("edge") < first_tick);
}

#[test]
fn a_range_outside_the_view_paints_nothing() {
    let shapes = recorded_shapes(&[(20.0, 30.0)]);
    let rects = UiProbe::rects(&shapes);
    assert!(rects.iter().all(|r| r.fill != <color::AccentSoft as ColorToken>::VALUE));
    assert!(rects.iter().all(|r| r.fill != <color::Accent as ColorToken>::VALUE));
}

#[test]
fn a_click_on_the_band_still_seeks() {
    let axis = TimeAxis { start: 0.0, seconds_per_point: 0.01, left: TrackHeader::WIDTH };
    let ranges = [(2.0, 4.0)];
    let at = egui::pos2(TrackHeader::WIDTH + 300.0, 20.0);
    let action = UiProbe::new(1490.0).click(at, |ui| Ruler::new(&axis, 10.0).recorded(&ranges).show(ui));
    assert!(matches!(action, Some(RulerAction::Seek(s)) if (s - 3.0).abs() < 1e-6), "{action:?}");
}

fn drag_frames(from: f32, to: f32) -> Vec<Option<RulerAction>> {
    let axis = TimeAxis { start: 0.0, seconds_per_point: 0.01, left: TrackHeader::WIDTH };
    UiProbe::new(1490.0).drag(egui::pos2(from, 20.0), egui::pos2(to, 20.0), |ui| Ruler::new(&axis, 10.0).show(ui))
}

#[test]
fn a_drag_on_the_ruler_scrubs_then_seeks_once() {
    let frames = drag_frames(TrackHeader::WIDTH + 100.0, TrackHeader::WIDTH + 500.0);

    let scrubs: Vec<f64> = frames
        .iter()
        .filter_map(|action| match action {
            Some(RulerAction::Scrub(seconds)) => Some(*seconds),
            _ => None,
        })
        .collect();
    assert!(scrubs.len() >= 2, "{frames:?}");
    assert!(scrubs.windows(2).all(|pair| pair[1] > pair[0]), "{scrubs:?}");
    let seeks: Vec<f64> = frames
        .iter()
        .filter_map(|action| match action {
            Some(RulerAction::Seek(seconds)) => Some(*seconds),
            _ => None,
        })
        .collect();
    assert_eq!(seeks.len(), 1, "{frames:?}");
    assert!((seeks[0] - 5.0).abs() < 1e-6, "{seeks:?}");
    assert!(matches!(frames.iter().rev().flatten().next(), Some(RulerAction::Seek(_))), "seek is last: {frames:?}");
}

#[test]
fn a_drag_that_ends_off_the_ruler_seeks_at_the_clamped_time() {
    let frames = drag_frames(TrackHeader::WIDTH + 100.0, 5000.0);

    let seeks: Vec<&Option<RulerAction>> =
        frames.iter().filter(|action| matches!(action, Some(RulerAction::Seek(_)))).collect();
    assert_eq!(seeks.len(), 1, "{frames:?}");
    assert!(matches!(seeks[0], Some(RulerAction::Seek(s)) if (*s - 10.0).abs() < 1e-6), "{seeks:?}");
}

#[test]
fn a_press_in_the_header_cell_does_nothing() {
    let frames = drag_frames(20.0, TrackHeader::WIDTH + 300.0);

    assert!(frames.iter().all(Option::is_none), "{frames:?}");
}
