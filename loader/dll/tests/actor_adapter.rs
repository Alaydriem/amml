use std::ffi::c_void;

use amml::{ActorAdapter, ClientAdapter, HandleStamp, SessionAdapter};
use amml_abi::{AmmlActorTable, AmmlApi, AmmlRotation, AmmlServices, AmmlStatus, AmmlVec3};

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
    assert_eq!(rows.len(), 2);
    assert_eq!((rows[0].id, rows[0].shape), (ActorAdapter::READ, Some("default")));
    assert_eq!((rows[1].id, rows[1].shape), (ActorAdapter::HEALTH, Some("default")));
    assert!(harness.state.areas().is_available(ActorAdapter::READ, 3));
    assert!(harness.state.areas().is_available(ActorAdapter::HEALTH, 1));
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
fn the_services_table_has_a_v3_actor_table() {
    let harness = Harness::start();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { &*AmmlApi::services(harness.state.api()).unwrap() };
    assert!(services.size >= AmmlServices::SIZE_V4);
    assert_eq!(table(&harness).size, AmmlActorTable::SIZE_V3);
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
