use std::ffi::c_void;

use amml::{ActorAdapter, ClientAdapter, HandleStamp, SessionAdapter};
use amml_abi::{
    AmmlAabb, AmmlActorTable, AmmlApi, AmmlGameType, AmmlPermissionLevel, AmmlRotation, AmmlServices, AmmlStatus,
    AmmlVec3,
};

use crate::support::{ActorGame, ClientGame, GAME_CLIENT_A, Harness, LogFile};

// Installs the session, client and actor adapters over every stand-in entry but `without`.
fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&ActorGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    harness
}

// `install` with every entry, after one frame and a joined world whose local player is the stand-in player.
fn start_in_world() -> Harness {
    let harness = install(&[]);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(ActorGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 1);
    harness
}

// The actor table, through the services table as a mod reaches it.
fn table(harness: &Harness) -> &'static AmmlActorTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::actor(services) }.unwrap();
    // SAFETY: as above.
    unsafe { &*table }
}

fn is_alive(table: &AmmlActorTable, player: usize) -> (AmmlStatus, u8) {
    let mut out = 0xEEu8;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.is_alive)(std::ptr::with_exposed_provenance::<c_void>(player), &mut out) };
    (status, out)
}

#[test]
fn the_area_is_offered_when_both_entries_resolve() {
    let harness = install(&[]);
    let rows = ActorAdapter::install(harness.state);
    assert_eq!(rows.len(), 5);
    assert_eq!((rows[0].id, rows[0].shape), (ActorAdapter::READ, Some("default")));
    assert_eq!((rows[1].id, rows[1].shape), (ActorAdapter::HEALTH, Some("default")));
    assert_eq!((rows[2].id, rows[2].shape), (ActorAdapter::GAME_TYPE, Some("default")));
    assert_eq!((rows[3].id, rows[3].shape), (ActorAdapter::PERMISSION, Some("getter")));
    assert_eq!((rows[4].id, rows[4].shape), (ActorAdapter::LOOK_WRITE, Some("default")));
    assert!(harness.state.areas().is_available(ActorAdapter::READ, 4));
    assert!(harness.state.areas().is_available(ActorAdapter::HEALTH, 1));
    assert!(harness.state.areas().is_available(ActorAdapter::GAME_TYPE, 1));
    assert!(harness.state.areas().is_available(ActorAdapter::PERMISSION, 1));
    assert!(harness.state.areas().is_available(ActorAdapter::LOOK_WRITE, 1));
}

#[test]
fn the_area_is_not_offered_without_the_alive_slot() {
    let harness = install(&["Actor.isAlive"]);
    assert!(!harness.state.areas().is_available(ActorAdapter::READ, 1));
    ClientGame::frame();
    ClientGame::set_player(ActorGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    let (status, _) = is_alive(table(&harness), ActorGame::player());
    assert_eq!(status, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn a_call_before_the_first_frame_is_not_ready() {
    let harness = install(&[]);
    assert_eq!(is_alive(table(&harness), ActorGame::player()).0, AmmlStatus::NOT_READY);
}

#[test]
fn the_local_player_reads_alive_then_dead() {
    let harness = start_in_world();
    let table = table(&harness);
    assert_eq!(is_alive(table, ActorGame::player()), (AmmlStatus::OK, 1));
    ActorGame::set_alive(false);
    assert_eq!(is_alive(table, ActorGame::player()), (AmmlStatus::OK, 0));
}

#[test]
fn a_handle_that_is_not_the_local_player_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    assert_eq!(is_alive(table, ActorGame::player() + 8).0, AmmlStatus::REFUSED);
    assert_eq!(is_alive(table, 0).0, AmmlStatus::REFUSED);
}

#[test]
fn a_call_off_the_game_thread_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let status = std::thread::scope(|scope| scope.spawn(|| is_alive(table, ActorGame::player()).0).join().unwrap());
    assert_eq!(status, AmmlStatus::REFUSED);
}

#[test]
fn a_null_out_is_an_invalid_argument() {
    let harness = start_in_world();
    // SAFETY: the table is live; the loader checks `out` before it writes.
    let status = unsafe { (table(&harness).is_alive)(std::ptr::null(), std::ptr::null_mut()) };
    assert_eq!(status, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn the_services_table_has_a_v6_actor_table() {
    let harness = Harness::start();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { &*AmmlApi::services(harness.state.api()).unwrap() };
    assert!(services.size >= AmmlServices::SIZE_V4);
    assert_eq!(table(&harness).size, AmmlActorTable::SIZE_V6);
}

fn point(read: Option<amml_abi::ActorPointFn>) -> (AmmlStatus, [f32; 3]) {
    let mut out = AmmlVec3 { x: 0.0, y: 0.0, z: 0.0 };
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (read.unwrap())(std::ptr::with_exposed_provenance(ActorGame::player()), &mut out) };
    (status, [out.x, out.y, out.z])
}

#[test]
fn the_eyes_are_mpos_and_the_feet_are_the_box_bottom() {
    let harness = start_in_world();
    let table = table(&harness);
    let eyes = ActorGame::eyes();
    assert_eq!(point(table.eye_position), (AmmlStatus::OK, eyes));
    assert_eq!(point(table.position), (AmmlStatus::OK, [eyes[0], ActorGame::feet_y(), eyes[2]]));
}

#[test]
fn the_rotation_is_mrot_pitch_then_yaw() {
    let harness = start_in_world();
    let mut out = AmmlRotation { pitch: 0.0, yaw: 0.0 };
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe {
        (table(&harness).rotation.unwrap())(std::ptr::with_exposed_provenance(ActorGame::player()), &mut out)
    };
    assert_eq!((status, [out.pitch, out.yaw]), (AmmlStatus::OK, ActorGame::rotation()));
}

#[test]
fn an_eye_read_logs_the_position_measurement_once_per_world() {
    let harness = start_in_world();
    let table = table(&harness);
    point(table.eye_position);
    point(table.eye_position);
    point(table.position);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "actor position measured");
    assert_eq!(log.matches("actor position measured").count(), 1, "{log}");
    assert!(log.contains("\"aabb_min_y\":64.0"), "{log}");
}

#[test]
fn the_head_is_the_attach_position_of_the_head_location() {
    let harness = start_in_world();
    assert_eq!(point(table(&harness).head_position), (AmmlStatus::OK, ActorGame::head()));
}

#[test]
fn the_block_source_is_the_one_of_the_dimension_of_the_player() {
    let harness = start_in_world();
    let mut out: *const c_void = std::ptr::null();
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe {
        (table(&harness).block_source.unwrap())(std::ptr::with_exposed_provenance(ActorGame::player()), &mut out)
    };
    // The handle is the block source with the epoch of the world (1) in its top bits.
    let sealed = HandleStamp::seal(ActorGame::block_source(), 1).unwrap();
    assert_eq!((status, out.addr()), (AmmlStatus::OK, sealed));
}

#[test]
fn a_head_or_block_source_read_of_another_handle_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let other = std::ptr::with_exposed_provenance::<c_void>(ActorGame::player() + 8);
    let mut head = AmmlVec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut source: *const c_void = std::ptr::null();
    // SAFETY: the table is live and both outputs are writable.
    let statuses = unsafe {
        [(table.head_position.unwrap())(other, &mut head), (table.block_source.unwrap())(other, &mut source)]
    };
    assert_eq!(statuses, [AmmlStatus::REFUSED, AmmlStatus::REFUSED]);
}

#[test]
fn the_health_is_read_through_the_entity_context() {
    let harness = start_in_world();
    let mut out = 0i32;
    // SAFETY: the table is live and `out` is writable.
    let status =
        unsafe { (table(&harness).health.unwrap())(std::ptr::with_exposed_provenance(ActorGame::player()), &mut out) };
    assert_eq!((status, out), (AmmlStatus::OK, ActorGame::HEALTH));
}

#[test]
fn the_health_area_is_not_offered_without_its_entries_and_the_read_gives_not_available() {
    let harness = install(&["ActorAttribute.getHealth"]);
    assert!(!harness.state.areas().is_available(ActorAdapter::HEALTH, 1));
    ClientGame::frame();
    ClientGame::set_player(ActorGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    let mut out = 0i32;
    // SAFETY: the table is live and `out` is writable.
    let status =
        unsafe { (table(&harness).health.unwrap())(std::ptr::with_exposed_provenance(ActorGame::player()), &mut out) };
    assert_eq!(status, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn without_the_block_source_entries_the_actor_area_is_not_offered_at_all() {
    let harness = install(&["Dimension.getBlockSourceFromMainChunkSource"]);
    assert!(!harness.state.areas().is_available(ActorAdapter::READ, 1));
    assert!(harness.state.areas().is_available(ActorAdapter::HEALTH, 1));
}

fn player_handle() -> *const c_void {
    std::ptr::with_exposed_provenance(ActorGame::player())
}

#[test]
fn the_view_vector_is_the_game_view_for_an_alpha_of_one() {
    let harness = start_in_world();
    assert_eq!(point(table(&harness).view_vector), (AmmlStatus::OK, ActorGame::view()));
}

#[test]
fn the_box_is_the_shape_component_min_then_max() {
    let harness = start_in_world();
    let mut out = AmmlAabb::default();
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table(&harness).aabb.unwrap())(player_handle(), &mut out) };
    let [min_x, min_y, min_z, max_x, max_y, max_z] = ActorGame::aabb();
    let expected =
        AmmlAabb { min: AmmlVec3 { x: min_x, y: min_y, z: min_z }, max: AmmlVec3 { x: max_x, y: max_y, z: max_z } };
    assert_eq!((status, out), (AmmlStatus::OK, expected));
}

#[test]
fn a_view_vector_or_box_read_of_another_handle_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let other = std::ptr::with_exposed_provenance::<c_void>(ActorGame::player() + 8);
    let mut view = AmmlVec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut shape = AmmlAabb::default();
    // SAFETY: the table is live and both outputs are writable.
    let statuses =
        unsafe { [(table.view_vector.unwrap())(other, &mut view), (table.aabb.unwrap())(other, &mut shape)] };
    assert_eq!(statuses, [AmmlStatus::REFUSED, AmmlStatus::REFUSED]);
}

fn game_type(harness: &Harness) -> (AmmlStatus, AmmlGameType) {
    let mut out = AmmlGameType(-99);
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table(harness).game_type.unwrap())(player_handle(), &mut out) };
    (status, out)
}

#[test]
fn the_game_type_is_the_raw_value_of_the_getter() {
    let harness = start_in_world();
    assert_eq!(game_type(&harness), (AmmlStatus::OK, AmmlGameType::CREATIVE));
    ActorGame::set_game_type(77);
    assert_eq!(game_type(&harness), (AmmlStatus::OK, AmmlGameType(77)));
}

#[test]
fn the_game_type_area_is_not_offered_without_its_getter() {
    let harness = start_in_world_without(&["Player.getPlayerGameType"]);
    assert!(!harness.state.areas().is_available(ActorAdapter::GAME_TYPE, 1));
    assert_eq!(game_type(&harness).0, AmmlStatus::NOT_AVAILABLE);
}

fn permission(harness: &Harness) -> (AmmlStatus, AmmlPermissionLevel) {
    let mut out = AmmlPermissionLevel(99);
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table(harness).permission_level.unwrap())(player_handle(), &mut out) };
    (status, out)
}

// Enters the world of the stand-in game over every entry but `without`.
fn start_in_world_without(without: &[&str]) -> Harness {
    let harness = install(without);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(ActorGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

#[test]
fn permission_shape_follows_the_resolved_entries() {
    // The getter of 26.63: its shape, and the level it gives.
    let harness = start_in_world_without(&["Player.getAbilitiesComponent"]);
    assert_eq!(permission(&harness), (AmmlStatus::OK, AmmlPermissionLevel::OPERATOR));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "actor permission read");
    assert!(log.contains("\"shape\":\"getter\""), "{log}");
}

#[test]
fn the_abilities_shape_reads_the_permission_byte_of_the_component() {
    ActorGame::set_permission(1);
    let harness = start_in_world_without(&["Player.getPlayerPermissionLevel"]);
    assert_eq!(permission(&harness), (AmmlStatus::OK, AmmlPermissionLevel::MEMBER));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "actor permission read");
    assert!(log.contains("\"shape\":\"abilities\""), "{log}");
    assert!(log.contains("\"value\":1"), "{log}");
}

#[test]
fn the_permission_area_is_not_offered_when_neither_shape_resolves() {
    let harness = start_in_world_without(&["Player.getPlayerPermissionLevel", "Player.getAbilitiesComponent"]);
    assert!(!harness.state.areas().is_available(ActorAdapter::PERMISSION, 1));
    assert_eq!(permission(&harness).0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn the_abilities_shape_needs_the_byte_offset() {
    let harness = start_in_world_without(&["Player.getPlayerPermissionLevel", "AbilitiesComponent.mPlayerPermissions"]);
    assert!(!harness.state.areas().is_available(ActorAdapter::PERMISSION, 1));
}

#[test]
fn the_permission_read_logs_once_per_world() {
    let harness = start_in_world();
    permission(&harness);
    permission(&harness);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "actor permission read");
    assert_eq!(log.matches("actor permission read").count(), 1, "{log}");
}

fn set_rotation(harness: &Harness, handle: *const c_void, rotation: AmmlRotation, head: Option<f32>) -> AmmlStatus {
    // SAFETY: the table is live and `rotation` is readable.
    unsafe { (table(harness).set_rotation.unwrap())(handle, &rotation, head.unwrap_or(0.0), u8::from(head.is_some())) }
}

#[test]
fn set_rotation_writes_current_and_previous() {
    let harness = start_in_world();
    let status = set_rotation(&harness, player_handle(), AmmlRotation { yaw: 12.5, pitch: -7.0 }, None);
    // `mRot` and the previous rotation take the pitch, then the yaw; the head is not touched.
    assert_eq!((status, ActorGame::rotation_component()), (AmmlStatus::OK, [-7.0, 12.5, -7.0, 12.5]));
    assert_eq!(ActorGame::head_rotations().1, 0);
}

#[test]
fn set_rotation_with_a_head_yaw_also_calls_the_head_setter_with_both_angles_equal() {
    let harness = start_in_world();
    let status = set_rotation(&harness, player_handle(), AmmlRotation { yaw: 40.0, pitch: 5.0 }, Some(33.0));
    assert_eq!((status, ActorGame::head_rotations()), (AmmlStatus::OK, ([33.0, 33.0], 1)));
}

#[test]
fn set_rotation_rejects_nan() {
    let harness = start_in_world();
    let before = ActorGame::rotation_component();
    for (rotation, head) in [
        (AmmlRotation { yaw: f32::NAN, pitch: 0.0 }, None),
        (AmmlRotation { yaw: 0.0, pitch: f32::INFINITY }, None),
        (AmmlRotation { yaw: 1.0, pitch: 1.0 }, Some(f32::NAN)),
    ] {
        assert_eq!(set_rotation(&harness, player_handle(), rotation, head), AmmlStatus::INVALID_ARGUMENT);
    }
    assert_eq!((ActorGame::rotation_component(), ActorGame::head_rotations().1), (before, 0));
}

#[test]
fn a_head_yaw_that_is_not_finite_is_ignored_without_the_flag() {
    let harness = start_in_world();
    // With the flag 0 the head value is not read.
    // SAFETY: the table is live and the rotation is readable.
    let status = unsafe {
        (table(&harness).set_rotation.unwrap())(player_handle(), &AmmlRotation { yaw: 2.0, pitch: 3.0 }, f32::NAN, 0)
    };
    assert_eq!((status, ActorGame::head_rotations().1), (AmmlStatus::OK, 0));
}

#[test]
fn set_rotation_of_a_stale_player_is_refused() {
    let harness = start_in_world();
    let before = ActorGame::rotation_component();
    // The world ended: the old handle is no longer the local player.
    ClientGame::set_player(0);
    let statuses = [
        set_rotation(&harness, player_handle(), AmmlRotation { yaw: 9.0, pitch: 9.0 }, Some(9.0)),
        set_rotation(&harness, std::ptr::null(), AmmlRotation { yaw: 9.0, pitch: 9.0 }, None),
    ];
    assert_eq!(statuses, [AmmlStatus::REFUSED, AmmlStatus::REFUSED]);
    assert_eq!((ActorGame::rotation_component(), ActorGame::head_rotations().1), (before, 0));
}

#[test]
fn set_rotation_off_the_game_thread_is_refused() {
    let harness = start_in_world();
    let before = ActorGame::rotation_component();
    let status = std::thread::scope(|scope| {
        scope
            .spawn(|| set_rotation(&harness, player_handle(), AmmlRotation { yaw: 9.0, pitch: 9.0 }, None))
            .join()
            .unwrap()
    });
    assert_eq!((status, ActorGame::rotation_component()), (AmmlStatus::REFUSED, before));
}

#[test]
fn the_look_write_area_is_not_offered_without_the_head_setter_and_the_write_gives_not_available() {
    let harness = start_in_world_without(&["Actor.setYHeadRotations"]);
    assert!(!harness.state.areas().is_available(ActorAdapter::LOOK_WRITE, 1));
    let before = ActorGame::rotation_component();
    let status = set_rotation(&harness, player_handle(), AmmlRotation { yaw: 9.0, pitch: 9.0 }, None);
    assert_eq!((status, ActorGame::rotation_component()), (AmmlStatus::NOT_AVAILABLE, before));
}
