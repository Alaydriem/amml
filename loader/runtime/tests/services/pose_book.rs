use amml_abi::{AmmlPose, AmmlRotation, AmmlVec3};
use amml_runtime::{PoseBook, PoseRefusal};

const OWNER: usize = 1;
const OTHER: usize = 2;

fn pose(x: f32) -> AmmlPose {
    AmmlPose {
        position: AmmlVec3 { x, y: 64.0, z: -2.5 },
        rotation: AmmlRotation { yaw: 30.0, pitch: 10.0 },
        roll: 0.0,
    }
}

#[test]
fn a_frame_without_set_pose_writes_nothing() {
    let book = PoseBook::new();
    assert!(book.begin_frame(1));
    book.set(OWNER, 1, pose(1.0)).unwrap();
    assert_eq!(book.pose(), Some(pose(1.0)));
    assert!(book.is_held());

    // The next frame has no call: nothing is left to write, so the game camera shows.
    assert!(book.begin_frame(2));

    assert_eq!(book.pose(), None);
    assert_eq!(book.write_position(), None);
    assert!(!book.is_held());
}

#[test]
fn a_second_setup_in_the_same_frame_keeps_the_pose() {
    let book = PoseBook::new();
    assert!(book.begin_frame(5));
    book.set(OWNER, 5, pose(1.0)).unwrap();

    assert!(!book.begin_frame(5));

    assert_eq!(book.pose(), Some(pose(1.0)));
}

#[test]
fn second_writer_in_a_frame_is_refused() {
    let book = PoseBook::new();
    book.begin_frame(1);
    book.set(OWNER, 1, pose(1.0)).unwrap();

    assert_eq!(book.set(OTHER, 1, pose(2.0)), Err(PoseRefusal::Taken));
    // The writer of the frame may change its own pose.
    book.set(OWNER, 1, pose(3.0)).unwrap();
    assert_eq!(book.pose(), Some(pose(3.0)));

    // In the next frame the other mod may write.
    book.begin_frame(2);
    book.set(OTHER, 2, pose(4.0)).unwrap();
    assert_eq!(book.pose(), Some(pose(4.0)));
}

#[test]
fn a_pose_for_another_frame_is_refused() {
    let book = PoseBook::new();
    book.begin_frame(3);

    assert_eq!(book.set(OWNER, 2, pose(1.0)), Err(PoseRefusal::OtherFrame));
    assert_eq!(book.set(OWNER, 4, pose(1.0)), Err(PoseRefusal::OtherFrame));
    assert_eq!(book.pose(), None);
}

#[test]
fn rendered_position_is_last_frames_pose() {
    let book = PoseBook::new();
    book.begin_frame(1);
    assert_eq!(book.rendered(), None);
    book.set(OWNER, 1, pose(7.0)).unwrap();
    assert_eq!(book.write_position(), Some(pose(7.0).position));
    // Inside the frame of the write, the rendered position is still the one of the frame before.
    assert_eq!(book.rendered(), None);

    book.begin_frame(2);
    assert_eq!(book.rendered(), Some(pose(7.0).position));

    // Frame 2 wrote nothing, so frame 3 has no rendered position.
    book.begin_frame(3);
    assert_eq!(book.rendered(), None);
}

#[test]
fn disable_drops_only_the_pose_of_that_mod() {
    let book = PoseBook::new();
    book.begin_frame(1);
    book.set(OWNER, 1, pose(1.0)).unwrap();

    book.disable_owner(OTHER);
    assert_eq!(book.pose(), Some(pose(1.0)));

    book.disable_owner(OWNER);
    assert_eq!(book.pose(), None);
    assert!(!book.is_held());
    // The frame is free again for another mod.
    book.set(OTHER, 1, pose(2.0)).unwrap();
}

#[test]
fn clear_forgets_the_pose_and_the_rendered_position() {
    let book = PoseBook::new();
    book.begin_frame(1);
    book.set(OWNER, 1, pose(1.0)).unwrap();
    book.write_position();
    book.begin_frame(2);
    book.set(OWNER, 2, pose(2.0)).unwrap();

    book.clear();

    assert_eq!(book.pose(), None);
    assert_eq!(book.rendered(), None);
    assert!(!book.is_held());
}
