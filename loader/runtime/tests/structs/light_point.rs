use amml_abi::AmmlLightFalloff;
use amml_runtime::LightPoint;

const ROUND: AmmlLightFalloff = AmmlLightFalloff::ROUND;
const TAXICAB: AmmlLightFalloff = AmmlLightFalloff::TAXICAB;

fn centred(block: [i32; 3], level: u8, falloff: AmmlLightFalloff) -> LightPoint {
    let position = block.map(|c| c as f32 + 0.5);
    LightPoint::at(position, level, falloff, 0).unwrap()
}

#[test]
fn full_level_at_the_source() {
    assert_eq!(centred([0, 64, 0], 14, ROUND).boost([0, 64, 0]), 14);
    assert_eq!(centred([0, 64, 0], 14, TAXICAB).boost([0, 64, 0]), 14);
}

#[test]
fn taxicab_matches_a_placed_torch() {
    let light = centred([0, 0, 0], 14, TAXICAB);
    assert_eq!(light.boost([1, 1, 0]), 12);
    assert_eq!(light.boost([13, 0, 0]), 1);
    assert_eq!(light.boost([14, 0, 0]), 0);
}

#[test]
fn round_uses_straight_line_distance() {
    let light = centred([0, 0, 0], 14, ROUND);
    assert_eq!(light.boost([1, 1, 0]), 13);
    assert_eq!(light.boost([5, 5, 5]), 5);
}

#[test]
fn an_unknown_falloff_reads_as_round() {
    let light = centred([0, 0, 0], 14, AmmlLightFalloff(9));
    assert_eq!(light.boost([1, 1, 0]), 13);
}

#[test]
fn a_quarter_step_moves_the_edge() {
    let centred = LightPoint::at([0.5, 0.5, 0.5], 10, ROUND, 0).unwrap();
    let forward = LightPoint::at([1.25, 0.5, 0.5], 10, ROUND, 0).unwrap();
    assert_eq!(centred.boost([3, 0, 0]), 7);
    assert_eq!(forward.boost([3, 0, 0]), 8);
    assert_eq!(forward.boost([-3, 0, 0]), 6);
}

#[test]
fn positions_in_the_same_quarter_block_are_one_light() {
    let a = LightPoint::at([1.26, 64.0, -0.1], 14, ROUND, 0);
    let b = LightPoint::at([1.49, 64.2, -0.2], 14, ROUND, 0);
    assert_eq!(a, b);
    assert_ne!(a, LightPoint::at([1.51, 64.0, -0.1], 14, ROUND, 0));
}

#[test]
fn far_blocks_get_nothing() {
    let light = centred([0, 0, 0], 15, ROUND);
    assert_eq!(light.boost([0, 40, 0]), 0);
    assert_eq!(light.boost([i32::MIN, 0, 0]), 0);
}

#[test]
fn a_bad_level_or_position_is_no_light() {
    assert_eq!(LightPoint::at([0.0; 3], 0, ROUND, 0), None);
    assert_eq!(LightPoint::at([0.0; 3], 16, ROUND, 0), None);
    assert_eq!(LightPoint::at([f32::NAN, 0.0, 0.0], 14, ROUND, 0), None);
    assert_eq!(LightPoint::at([f32::INFINITY, 0.0, 0.0], 14, ROUND, 0), None);
    assert_eq!(LightPoint::at([3.0e9, 0.0, 0.0], 14, ROUND, 0), None);
}

#[test]
fn negative_positions_floor() {
    let light = LightPoint::at([-0.1, 8.0, 8.0], 1, ROUND, 0).unwrap();
    assert_eq!(light.block_pos(), [-1, 8, 8]);
    assert_eq!(light.subchunk(), [-1, 0, 0]);
}

#[test]
fn the_reach_box_is_the_block_plus_and_minus_the_level() {
    assert_eq!(centred([0, 64, 0], 15, ROUND).reach_box(), ([-15, 49, -15], [15, 79, 15]));
}

#[test]
fn reaches_the_subchunks_in_range_only() {
    let light = centred([2, 64, 15], 14, ROUND);
    assert!(light.reaches_subchunk([0, 4, 0]));
    assert!(light.reaches_subchunk([-1, 4, 1]));
    assert!(light.reaches_subchunk([0, 3, 0]));
    assert!(!light.reaches_subchunk([2, 4, 0]));
    assert!(!light.reaches_subchunk([0, 5, 0]));
    assert!(!light.reaches_subchunk([0, 4, 2]));
}
