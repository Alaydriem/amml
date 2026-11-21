use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::AmmlLightFalloff;
use amml_runtime::{LightCell, LightPoint, VvPlan};

fn centred(block: [i32; 3], level: u8, block_ptr: usize) -> LightPoint {
    LightPoint::at(block.map(|c| c as f32 + 0.5), level, AmmlLightFalloff::ROUND, block_ptr).unwrap()
}

#[test]
fn no_light_keeps_the_game_value() {
    let cell = LightCell::new();
    assert!(!cell.is_lit());
    assert_eq!(cell.boost_at([0, 0, 0], 2), 2);
}

#[test]
fn raises_the_block_byte_in_reach_only() {
    let cell = LightCell::new();
    cell.publish(&[centred([0, 0, 0], 14, 0)]);
    assert_eq!(cell.raise_pair([0, 2], [0, 0, 0]), [0, 14]);
    assert_eq!(cell.raise_pair([0, 2], [0, 0, 3]), [0, 11]);
    assert_eq!(cell.raise_pair([0, 2], [0, 0, 20]), [0, 2]);
}

#[test]
fn keeps_a_brighter_game_value_and_never_changes_sky() {
    let cell = LightCell::new();
    cell.publish(&[centred([0, 0, 0], 14, 0)]);
    assert_eq!(cell.raise_pair([7, 15], [0, 0, 3]), [7, 15]);
}

#[test]
fn several_lights_take_the_max_per_block() {
    let cell = LightCell::new();
    cell.publish(&[centred([0, 0, 0], 5, 0), centred([4, 0, 0], 14, 0)]);
    assert_eq!(cell.boost_at([0, 0, 0], 0), 10);
    assert_eq!(cell.boost_at([-4, 0, 0], 0), 6);
}

#[test]
fn publishing_nothing_clears() {
    let cell = LightCell::new();
    cell.publish(&[centred([0, 0, 0], 14, 0)]);
    cell.publish(&[]);
    assert!(!cell.is_lit());
    assert_eq!(cell.boost_at([0, 0, 0], 2), 2);
}

#[test]
fn vv_plan_skips_in_reach_without_an_emitter() {
    let cell = LightCell::new();
    assert_eq!(cell.vv_plan([0, 4, 0], 0, true), VvPlan::Normal);
    cell.publish(&[centred([2, 64, 15], 14, 0)]);
    assert_eq!(cell.vv_plan([0, 4, 0], 0, true), VvPlan::Skip);
    assert_eq!(cell.vv_plan([5, 4, 0], 0, true), VvPlan::Normal);
}

#[test]
fn vv_plan_emits_near_the_light_with_a_block() {
    let cell = LightCell::new();
    cell.publish(&[centred([2, 64, 15], 14, 0x1000)]);
    let emit = VvPlan::Emit { block: 0x1000, position: [2, 64, 15], emission: 14 };
    assert_eq!(cell.vv_plan([0, 4, 0], 0, true), emit);
    assert_eq!(cell.vv_plan([1, 5, -1], 0, true), emit);
    assert_eq!(cell.vv_plan([2, 4, 0], 0, true), VvPlan::Normal);
}

#[test]
fn vv_plan_skips_when_the_emitter_cannot_run() {
    let cell = LightCell::new();
    cell.publish(&[centred([2, 64, 15], 14, 0x1000)]);
    assert_eq!(cell.vv_plan([0, 4, 0], 0, false), VvPlan::Skip);
    assert_eq!(cell.vv_plan([0, 4, 0], 1, true), VvPlan::Skip);
}

#[test]
fn a_reader_never_sees_a_mixed_light() {
    let cell = Arc::new(LightCell::new());
    let stop = Arc::new(AtomicBool::new(false));
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let cell = Arc::clone(&cell);
            let stop = Arc::clone(&stop);
            std::thread::spawn(move || {
                let mut mixed = 0;
                while !stop.load(Ordering::Relaxed) {
                    let (points, count) = cell.read();
                    for point in &points[..count] {
                        let [x, y, z] = point.steps;
                        if x != y || y != z || i64::from(point.level) != x.rem_euclid(15) + 1 {
                            mixed += 1;
                        }
                    }
                }
                mixed
            })
        })
        .collect();
    for i in 0..200_000i64 {
        let level = u8::try_from(i.rem_euclid(15) + 1).unwrap();
        let point = LightPoint { steps: [i; 3], level, falloff: AmmlLightFalloff::ROUND, block: 0 };
        cell.publish(&[point, point]);
    }
    stop.store(true, Ordering::Relaxed);
    for reader in readers {
        assert_eq!(reader.join().unwrap(), 0);
    }
}
