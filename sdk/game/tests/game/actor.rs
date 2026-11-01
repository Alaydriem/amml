use std::ffi::c_void;

use amml_game::{Actor, ActorPose, EntryId};

use crate::support::{FakeActor, FakeDimension, FakeRotation, FakeStateVector};

#[test]
fn pose_reads_position_pitch_and_yaw_through_the_two_component_pointers() {
    let entries = FakeActor::entries();
    let mut state_vector = FakeStateVector::new(1.5, 64.0, -3.25);
    let mut rotation = FakeRotation::new(10.0, -90.0);
    let mut fake = FakeActor::new(
        (&raw mut state_vector).cast(),
        (&raw mut rotation).cast(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    // SAFETY: a live fake actor with the layout the entries name.
    let actor = unsafe { Actor::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(actor.pose(), Some(ActorPose { pos: [1.5, 64.0, -3.25], pitch: 10.0, yaw: -90.0 }));
}

#[test]
fn a_null_component_pointer_anywhere_on_the_path_gives_none_and_no_read_follows() {
    let entries = FakeActor::entries();
    let mut state_vector = FakeStateVector::new(1.5, 64.0, -3.25);
    let mut rotation = FakeRotation::new(10.0, -90.0);

    // A read through a null component pointer would fault at a small address, so a pass shows no read followed.
    let mut no_rotation = FakeActor::new(
        (&raw mut state_vector).cast(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    // SAFETY: a live fake actor with the layout the entries name.
    let actor = unsafe { Actor::from_raw(no_rotation.ptr(), &entries) }.unwrap();
    assert_eq!(actor.pose(), None);

    let mut no_state_vector =
        FakeActor::new(std::ptr::null_mut(), (&raw mut rotation).cast(), std::ptr::null_mut(), std::ptr::null_mut());
    // SAFETY: as above.
    let actor = unsafe { Actor::from_raw(no_state_vector.ptr(), &entries) }.unwrap();
    assert_eq!(actor.pose(), None);
}

#[test]
fn pose_is_none_when_a_layout_entry_is_missing() {
    let entries = FakeActor::entries_without(EntryId::ACTOR_ROTATION_COMPONENT_ROT);
    let mut state_vector = FakeStateVector::new(1.5, 64.0, -3.25);
    let mut rotation = FakeRotation::new(10.0, -90.0);
    let mut fake = FakeActor::new(
        (&raw mut state_vector).cast(),
        (&raw mut rotation).cast(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    // SAFETY: a live fake actor with the layout the entries name.
    let actor = unsafe { Actor::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(actor.pose(), None);
}

#[test]
fn dimension_and_block_source_come_from_the_game_getters_and_null_gives_none() {
    let entries = FakeActor::entries();
    let mut dimension = FakeDimension::new(7);
    let source = 0x8000 as *mut c_void;
    let mut fake = FakeActor::new(std::ptr::null_mut(), std::ptr::null_mut(), dimension.ptr(), source);
    // SAFETY: a live fake actor with the layout the entries name.
    let actor = unsafe { Actor::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(actor.dimension().and_then(|dimension| dimension.id()), Some(7));
    assert_eq!(actor.block_source().map(|source| source.as_ptr()), Some(source));

    let mut out_of_world =
        FakeActor::new(std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
    // SAFETY: as above.
    let actor = unsafe { Actor::from_raw(out_of_world.ptr(), &entries) }.unwrap();
    assert!(actor.dimension().is_none());
    assert!(actor.block_source().is_none());
}

#[test]
fn level_reads_the_level_pointer() {
    let entries = FakeActor::entries();
    let mut fake =
        FakeActor::new(std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
    // SAFETY: a live fake actor with the layout the entries name.
    let actor = unsafe { Actor::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(actor.level().map(|level| level.as_ptr() as usize), Some(0x4000));
}
