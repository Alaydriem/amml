use amml_studio_ui::component::viewer::{PathMap, PathMapAction, PathMapCamera, PathMapPoint, PathMapView};
use amml_studio_ui::tokens::selector::SceneColor;
use amml_ui::tokens::ColorToken;
use amml_ui::tokens::color;
use egui::{Pos2, Rect, Shape, pos2};

use crate::glyph::words;
use crate::support::UiProbe;

fn view(path: Vec<PathMapPoint>, players: Vec<Pos2>, camera: Pos2) -> PathMapView {
    PathMapView {
        label: words("Path · Arrival"),
        scene: SceneColor::Lime,
        path,
        hidden: false,
        camera: PathMapCamera { at: camera, heading: 1.0 },
        scene_drives: false,
        players,
    }
}

// shows the map at the top left and returns its rect, the action and what it painted
fn show(view: &PathMapView) -> (Rect, Vec<Shape>) {
    let mut probe = UiProbe::new(400.0);
    probe.frame(Vec::new(), |ui| PathMap::new(view).show(ui));
    let ((_, rect), shapes) = probe.frame(Vec::new(), |ui| (PathMap::new(view).show(ui), ui.min_rect()));
    (rect, shapes)
}

#[test]
fn path_map_with_no_points() {
    let view = view(Vec::new(), Vec::new(), pos2(0.0, 0.0));
    let (rect, shapes) = show(&view);
    let playhead = <color::Playhead as ColorToken>::VALUE;
    let cone = shapes.iter().any(|shape| matches!(shape, Shape::Circle(dot) if dot.fill == playhead));
    assert!(cone, "the camera dot is painted");
    assert_eq!(rect.size(), PathMap::SIZE);
}

#[test]
fn path_map_fits_the_points() {
    let spread = |index: usize| pos2((index * 1_237 % 10_000) as f32, (index * 7_919 % 10_000) as f32 - 5_000.0);
    let path = (0..40).map(|index| PathMapPoint { at: spread(index), selected: index == 3 }).collect();
    let players = (40..60).map(spread).collect();
    let view = view(path, players, pos2(10_000.0, -5_000.0));
    let (rect, shapes) = show(&view);
    let mut checked = 0;
    for shape in &shapes {
        let points: Vec<Pos2> = match shape {
            Shape::Circle(circle) => vec![circle.center],
            Shape::Path(path) => path.points.clone(),
            _ => continue,
        };
        for point in points {
            assert!(rect.contains(point), "{point:?} is outside {rect:?}");
            checked += 1;
        }
    }
    assert!(checked > 60, "only {checked} points were painted");
}

#[test]
fn waypoint_click_selects_the_key() {
    let path = vec![
        PathMapPoint { at: pos2(0.0, 0.0), selected: false },
        PathMapPoint { at: pos2(100.0, 50.0), selected: false },
    ];
    let view = view(path, Vec::new(), pos2(50.0, 25.0));
    let mut probe = UiProbe::new(400.0);
    let ((_, places), _) = probe.frame(Vec::new(), |ui| {
        let action = PathMap::new(&view).show(ui);
        (action, PathMap::new(&view).waypoints(ui.min_rect()))
    });
    let action = probe.click(places[1], |ui| PathMap::new(&view).show(ui));
    assert_eq!(action, Some(PathMapAction::SelectKey(1)));
}
