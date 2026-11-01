use amml_studio_ui::component::timeline::{BoxSelect, KeyKind, LaneKey, TimeAxis};
use egui::{Rect, pos2, vec2};

#[test]
fn box_select_of_many_keys() {
    let axis = TimeAxis { start: 0.0, seconds_per_point: 0.1, left: 222.0 };
    let lanes: Vec<(Rect, Vec<LaneKey>)> = (0..10)
        .map(|lane| {
            let rect = Rect::from_min_size(pos2(222.0, 100.0 + 64.0 * lane as f32), vec2(1268.0, 64.0));
            let keys = (0..200)
                .map(|index| LaneKey {
                    seconds: (index * 7 + lane * 3) as f64 * 0.09,
                    kind: KeyKind::Eased,
                    selected: false,
                })
                .collect();
            (rect, keys)
        })
        .collect();
    // half the time range, over lanes 2 to 6
    let drag = Rect::from_min_max(
        pos2(222.0 + 300.0, 100.0 + 64.0 * 2.0 + 10.0),
        pos2(222.0 + 934.0, 100.0 + 64.0 * 7.0 - 10.0),
    );
    let borrowed: Vec<(Rect, &[LaneKey])> = lanes.iter().map(|(rect, keys)| (*rect, keys.as_slice())).collect();
    let picked = BoxSelect::new(drag).pick(&axis, &borrowed);
    let mut expected = Vec::new();
    for (lane, (rect, keys)) in lanes.iter().enumerate() {
        for (index, key) in keys.iter().enumerate() {
            if drag.contains(pos2(axis.x_of(key.seconds), rect.center().y)) {
                expected.push((lane, index));
            }
        }
    }
    assert!(expected.len() > 400, "the box covers many keys");
    assert_eq!(picked, expected);
}
