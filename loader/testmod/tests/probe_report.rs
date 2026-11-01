use amml_game::{Aabb, ActorPose, Brightness};
use amml_testmod::{ChunkReading, LightReading, NameTagLayout, Probe, ProbeReport};

#[test]
fn a_world_line_holds_the_position_angles_dimension_and_block_source() {
    let pose = ActorPose { pos: [1.0, 64.5, -3.25], pitch: -12.5, yaw: 90.0 };
    let line = ProbeReport::world_line(Some(pose), Some(1), Some(true));
    assert_eq!(line, "probe: pos=[1.00, 64.50, -3.25] pitch=-12.50 yaw=90.00 dimension=1 block_source=ok");
}

#[test]
fn a_world_line_marks_every_value_that_could_not_be_read() {
    let line = ProbeReport::world_line(None, None, None);
    assert_eq!(line, "probe: pos=n/a pitch=n/a yaw=n/a dimension=n/a block_source=n/a");
}

#[test]
fn a_camera_line_names_the_value_and_marks_a_missing_read() {
    assert_eq!(
        ProbeReport::camera_line("forward", Some([0.0, 0.0, 1.0])),
        "probe: camera forward=[0.000, 0.000, 1.000]"
    );
    assert_eq!(ProbeReport::camera_line("view_position", None), "probe: camera view_position=n/a");
}

#[test]
fn the_features_line_lists_each_off_feature_with_its_cause() {
    assert_eq!(ProbeReport::features_line(&[]), "probe: features off: none");
    let off = vec![("camera", "Camera.mForward: entry did not resolve".to_string()), ("pose", "x: y".to_string())];
    assert_eq!(
        ProbeReport::features_line(&off),
        "probe: features off: camera (Camera.mForward: entry did not resolve); pose (x: y)"
    );
}

#[test]
fn a_block_line_holds_the_position_name_and_solid_flag() {
    assert_eq!(
        ProbeReport::block_line([10, 63, -4], Some("minecraft:stone"), Some(true)),
        "probe: block pos=[10, 63, -4] name=minecraft:stone solid=true"
    );
    assert_eq!(ProbeReport::block_line([0, 0, 0], None, None), "probe: block pos=[0, 0, 0] name=n/a solid=n/a");
}

#[test]
fn a_chunk_line_holds_the_chunk_bounds_and_the_column_height() {
    let reading = ChunkReading {
        pos: Some([-1, 2]),
        min_y: Some(-64),
        max_y: Some(319),
        loaded: true,
        sub_chunks: 24,
        index: 37,
        height: Some(127),
    };
    assert_eq!(
        ProbeReport::chunk_line(&reading),
        "probe: chunk pos=[-1, 2] min_y=-64 max_y=319 loaded=true sub_chunks=24 height[37]=127"
    );
}

#[test]
fn a_block_y_outside_the_chunk_range_is_not_read() {
    assert!(ChunkReading::contains_y(-64, Some(319), -64));
    assert!(ChunkReading::contains_y(-64, Some(319), 319));
    assert!(!ChunkReading::contains_y(-64, Some(319), -65));
    assert!(!ChunkReading::contains_y(-64, Some(319), 320));
    assert!(!ChunkReading::contains_y(-64, None, 0));
    assert_eq!(ProbeReport::block_out_of_range_line([1, -70, 2]), "probe: block pos=[1, -70, 2] block=out of range");
}

#[test]
fn the_block_under_the_feet_is_read_from_the_eye_position() {
    assert_eq!(Probe::block_under_feet([1.5, 64.62, -0.5]), [1, 62, -1]);
    assert_eq!(Probe::block_under_feet([0.0, 65.1, 3.9]), [0, 62, 3]);
    assert_eq!(Probe::block_under_feet([-16.2, 64.0, 0.0]), [-17, 61, 0]);
}

#[test]
fn a_light_line_holds_the_light_the_support_flag_and_the_shape() {
    let reading = LightReading {
        feet: [4, 65, -2],
        brightness: Some(Brightness { sky: 15, block: 7 }),
        under_air: Some(false),
        under_full_support: Some(true),
        feet_air: Some(false),
        feet_shape: Some(Aabb { min: [0.0, 0.0, 0.0], max: [1.0, 0.5, 1.0] }),
        sky_darken: Some(3),
    };
    assert_eq!(
        ProbeReport::light_line(&reading),
        "probe: light feet=[4, 65, -2] sky=15 block=7 sky_darken=3 under_air=false under_full_support=true feet_air=false feet_shape=[0.00, 0.00, 0.00]-[1.00, 0.50, 1.00] empty=false"
    );
}

#[test]
fn a_light_line_marks_every_value_that_could_not_be_read() {
    let reading = LightReading {
        feet: [0, 0, 0],
        brightness: None,
        under_air: None,
        under_full_support: None,
        feet_air: None,
        feet_shape: None,
        sky_darken: None,
    };
    assert_eq!(
        ProbeReport::light_line(&reading),
        "probe: light feet=[0, 0, 0] sky=n/a block=n/a sky_darken=n/a under_air=n/a under_full_support=n/a feet_air=n/a feet_shape=n/a"
    );
}

#[test]
fn a_name_tags_line_gives_the_count_or_the_fault() {
    let good = NameTagLayout { first: 0x1000, last: 0x1000 + 2 * 144, end: 0x2000, element_size: 144 };
    assert_eq!(ProbeReport::name_tags_line(Some(&good)), "probe: name_tags count=2 layout=ok");

    let zero = NameTagLayout { first: 0x1000, last: 0x1000, end: 0x2000, element_size: 0 };
    assert_eq!(ProbeReport::name_tags_line(Some(&zero)), "probe: name_tags layout=broken (element size is 0)");

    assert_eq!(ProbeReport::name_tags_line(None), "probe: name_tags n/a");
}
