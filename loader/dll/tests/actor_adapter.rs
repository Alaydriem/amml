use std::ffi::c_void;

use amml::{ActorAdapter, ClientAdapter, SessionAdapter};
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
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].shape, Some("default"));
    assert!(harness.state.areas().is_available(ActorAdapter::READ, 1));
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
fn the_services_table_has_a_v2_actor_table() {
    let harness = Harness::start();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { &*AmmlApi::services(harness.state.api()).unwrap() };
    assert!(services.size >= AmmlServices::SIZE_V4);
    assert_eq!(table(&harness).size, AmmlActorTable::SIZE_V2);
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
