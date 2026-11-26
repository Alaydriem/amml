use amml_abi::AmmlLightFalloff;
use amml_runtime::{LightChange, LightDiff, LightPoint, VvPlan};

fn at(position: [f32; 3], level: u8, falloff: AmmlLightFalloff, block: usize) -> LightPoint {
    LightPoint::at(position, level, falloff, block).unwrap()
}

fn round(position: [f32; 3], level: u8) -> LightPoint {
    at(position, level, AmmlLightFalloff::ROUND, 0)
}

fn lit(points: &[LightPoint], pos: [i32; 3]) -> u8 {
    points.iter().map(|point| point.boost(pos)).max().unwrap_or(0)
}

// Checks the rule of the diff by brute force: every block whose light differs between `old` and `new`, and every
// block next to one (a mesh reads its neighbours), lies in a marked subchunk; every subchunk whose Vibrant
// Visuals plan differs is marked. Gives the change.
fn assert_covers(old: &[LightPoint], new: &[LightPoint]) -> LightChange {
    let change = LightDiff::new().change(old, new);
    let mut low = [i32::MAX; 3];
    let mut high = [i32::MIN; 3];
    for (min, max) in old.iter().chain(new).map(LightPoint::reach_box) {
        for axis in 0..3 {
            low[axis] = low[axis].min(min[axis] - 2);
            high[axis] = high[axis].max(max[axis] + 2);
        }
    }
    for x in low[0]..=high[0] {
        for y in low[1]..=high[1] {
            for z in low[2]..=high[2] {
                if lit(old, [x, y, z]) == lit(new, [x, y, z]) {
                    continue;
                }
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            let near = [x + dx, y + dy, z + dz].map(|c| c.div_euclid(16));
                            assert!(
                                change.contains(near),
                                "block {:?} differs, subchunk {near:?} not marked",
                                [x, y, z]
                            );
                        }
                    }
                }
            }
        }
    }
    for sx in low[0].div_euclid(16) - 1..=high[0].div_euclid(16) + 1 {
        for sy in low[1].div_euclid(16) - 1..=high[1].div_euclid(16) + 1 {
            for sz in low[2].div_euclid(16) - 1..=high[2].div_euclid(16) + 1 {
                for (variant, ready) in [(0, false), (0, true), (1, false), (1, true), (7, true)] {
                    let subchunk = [sx, sy, sz];
                    if VvPlan::of(old, subchunk, variant, ready) != VvPlan::of(new, subchunk, variant, ready) {
                        assert!(change.contains(subchunk), "plan of {subchunk:?} differs, not marked");
                    }
                }
            }
        }
    }
    change
}

#[test]
fn every_differing_block_of_a_walk_is_marked() {
    for level in [15u8, 7] {
        let mut old: Option<LightPoint> = None;
        for tick in 0..200 {
            let along = 0.215 * tick as f32;
            let new = round([-10.3 + 0.96 * along, 70.62, 5.1 + 0.28 * along], level);
            if old != Some(new) {
                assert_covers(old.as_slice(), &[new]);
            }
            old = Some(new);
        }
    }
}

#[test]
fn a_light_crossing_a_subchunk_border_marks_both_sides() {
    let old = round([15.9, 64.1, 31.9], 7);
    let new = round([16.1, 63.9, 32.1], 7);
    let change = assert_covers(&[old], &[new]);
    assert!(change.contains([0, 3, 1]) && change.contains([1, 4, 2]));
}

#[test]
fn negative_coordinates_are_covered() {
    assert_covers(&[round([-0.1, -0.1, -15.9], 11)], &[round([0.1, -0.4, -16.1], 11)]);
}

#[test]
fn two_lights_are_covered_when_one_moves() {
    let still = round([4.5, 70.5, 4.5], 15);
    let old = round([8.2, 70.6, 4.5], 9);
    let new = round([8.45, 70.6, 4.5], 9);
    assert_covers(&[still, old], &[still, new]);
    assert_covers(&[old, still], &[new, still]);
}

#[test]
fn an_unchanged_light_set_marks_nothing() {
    let lights = [round([4.5, 70.5, 4.5], 15), round([30.5, 70.5, 4.5], 4)];
    let change = LightDiff::new().change(&lights, &[lights[1], lights[0]]);
    assert!(change.is_empty());
    assert_eq!(change.evaluations(), 0);
}

#[test]
fn a_light_set_or_cleared_marks_all_it_lights() {
    let light = round([0.5, 64.5, 0.5], 12);
    let set = assert_covers(&[], &[light]);
    let cleared = assert_covers(&[light], &[]);
    assert_eq!(set, cleared);
}

#[test]
fn a_level_or_falloff_change_is_covered() {
    let old = round([3.3, 64.5, 9.7], 14);
    assert_covers(&[old], &[round([3.3, 64.5, 9.7], 10)]);
    assert_covers(&[old], &[at([3.3, 64.5, 9.7], 14, AmmlLightFalloff::TAXICAB, 0)]);
}

#[test]
fn a_block_light_moving_covers_the_emitter_plans() {
    // Its emitter plan follows the block position; a move across a subchunk border changes the plans one
    // subchunk around both positions.
    let old = at([15.6, 70.5, 8.5], 3, AmmlLightFalloff::ROUND, 0x1000);
    let new = at([16.1, 70.5, 8.5], 3, AmmlLightFalloff::ROUND, 0x1000);
    let change = assert_covers(&[old], &[new]);
    assert!(change.contains([-1, 4, 0]) && change.contains([2, 4, 0]));
}

#[test]
fn boxes_hold_each_marked_subchunk_once_in_x_runs() {
    let change = LightDiff::new().change(&[], &[round([16.5, 72.5, 8.5], 9)]);
    let mut covered = Vec::new();
    for (low, high) in change.boxes() {
        assert_eq!([low[1], low[2], high[1], high[2]].map(|c| c.rem_euclid(16)), [0, 0, 15, 15]);
        assert_eq!((low[1], low[2]), (high[1] - 15, high[2] - 15));
        for x in low[0].div_euclid(16)..=high[0].div_euclid(16) {
            covered.push([x, low[1].div_euclid(16), low[2].div_euclid(16)]);
        }
    }
    let count = covered.len();
    covered.sort_unstable();
    covered.dedup();
    assert_eq!((covered.len(), count), (change.marks(), change.marks()));
    assert!(change.boxes().len() < change.marks());
}

#[test]
fn a_block_only_change_scans_no_block() {
    let old = at([4.3, 70.6, -2.2], 15, AmmlLightFalloff::ROUND, 0x1000);
    let new = at([4.3, 70.6, -2.2], 15, AmmlLightFalloff::ROUND, 0x2000);
    let change = assert_covers(&[old], &[new]);
    assert_eq!(change.evaluations(), 0);
    // Only the emitter plans differ: the 27 subchunks around the light.
    assert_eq!(change.marks(), 27);
}

// The most evaluations of one change over a sweep of level-15 changes: one quarter-block step along each axis from
// each of the 64 quarter positions in a block, a level change, a falloff change, and four lights cleared at once.
#[test]
fn the_worst_change_of_a_sweep_stays_bounded() {
    let diff = LightDiff::new();
    let mut worst = (0u64, String::new());
    let mut note = |change: LightChange, what: String| {
        if change.evaluations() > worst.0 {
            worst = (change.evaluations(), what);
        }
    };
    for offset in 0..64 {
        let base = [offset % 4, (offset / 4) % 4, offset / 16].map(|q| q as f32 * 0.25 + 0.1);
        let position = [base[0] + 7.0, base[1] + 70.0, base[2] - 3.0];
        let old = round(position, 15);
        for axis in 0..3 {
            for step in [-0.25f32, 0.25] {
                let mut moved = position;
                moved[axis] += step;
                note(diff.change(&[old], &[round(moved, 15)]), format!("step {axis} {step} from {position:?}"));
            }
        }
        note(diff.change(&[old], &[round(position, 14)]), format!("level from {position:?}"));
        let taxicab = at(position, 15, AmmlLightFalloff::TAXICAB, 0);
        note(diff.change(&[old], &[taxicab]), format!("falloff at {position:?}"));
    }
    let four = [
        round([0.5, 64.5, 0.5], 15),
        round([40.5, 64.5, 0.5], 15),
        round([0.5, 64.5, 40.5], 15),
        round([40.5, 90.5, 40.5], 15),
    ];
    note(diff.change(&four, &[]), "four cleared".to_string());
    // The limit is the measured worst (3180); raise it with a new measurement when a correct change needs more.
    assert!(worst.0 <= 3180, "worst {worst:?}");
}
