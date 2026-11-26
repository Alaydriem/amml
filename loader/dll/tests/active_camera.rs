use std::ffi::c_void;

use amml::{ActiveCameraAdapter, SessionAdapter};
use amml_abi::{AmmlApi, AmmlCameraKind, AmmlCameraTable, AmmlServices, AmmlStatus};

use crate::support::{ActiveCameraGame, ClientGame, GAME_CLIENT_A, GAME_CLIENT_B, Harness, LogFile, SceneCamera};

// The adapters over the stand-in client and camera registry, in the shape of a build (`size_shape` is 70.26), with the
// component size entries' value replaced by `size`; after one frame and one update of client A (the primary client).
fn install(size_shape: bool, size: Option<u64>, cameras: &[SceneCamera]) -> Harness {
    let mut entries = ClientGame::entries(&[]);
    entries.extend(ActiveCameraGame::entries(size_shape, size));
    let harness = Harness::start_with_entries(&entries);
    ActiveCameraGame::build(size_shape, cameras);
    SessionAdapter::install(harness.state);
    ActiveCameraAdapter::install(harness.state);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

// `install` for the real shape of a build, then a joined world (epoch 1).
fn start_in_world(size_shape: bool, cameras: &[SceneCamera]) -> Harness {
    let harness = install(size_shape, None, cameras);
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 1);
    harness
}

fn table(harness: &Harness) -> AmmlCameraTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::camera(services) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlCameraTable::read(table) }.unwrap()
}

fn handle() -> *const c_void {
    ClientGame::client(&GAME_CLIENT_A).cast_const()
}

fn active(table: &AmmlCameraTable, client: *const c_void) -> (AmmlStatus, AmmlCameraKind) {
    let mut out = AmmlCameraKind(0);
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.active_camera.unwrap())(client, &raw mut out) };
    (status, out)
}

fn scene() -> [SceneCamera; 4] {
    [
        SceneCamera::named("minecraft:first_person", false),
        SceneCamera::named("minecraft:third_person", true),
        SceneCamera::named("minecraft:third_person_front", false),
        SceneCamera::named("minecraft:free", false),
    ]
}

// The kind the table gives when the camera named `active_name` is the active one, on the layout of a build. One
// process holds one loader state, so each layout and name is its own test.
fn kind_is(size_shape: bool, active_name: &str, kind: AmmlCameraKind) {
    let cameras = scene().map(|camera| SceneCamera { active: camera.name == active_name, ..camera });
    let harness = start_in_world(size_shape, &cameras);

    assert_eq!(active(&table(&harness), handle()), (AmmlStatus::OK, kind));
}

#[test]
fn first_person_is_read_on_the_26_63_layout() {
    kind_is(false, "minecraft:first_person", AmmlCameraKind::FIRST_PERSON);
}

#[test]
fn third_person_is_read_on_the_26_63_layout() {
    kind_is(false, "minecraft:third_person", AmmlCameraKind::THIRD_PERSON);
}

#[test]
fn third_person_front_is_read_on_the_26_63_layout() {
    kind_is(false, "minecraft:third_person_front", AmmlCameraKind::THIRD_PERSON_FRONT);
}

#[test]
fn another_camera_is_other_on_the_26_63_layout() {
    kind_is(false, "minecraft:free", AmmlCameraKind::OTHER);
}

#[test]
fn first_person_is_read_on_the_70_26_layout() {
    kind_is(true, "minecraft:first_person", AmmlCameraKind::FIRST_PERSON);
}

#[test]
fn third_person_is_read_on_the_70_26_layout() {
    kind_is(true, "minecraft:third_person", AmmlCameraKind::THIRD_PERSON);
}

#[test]
fn third_person_front_is_read_on_the_70_26_layout() {
    kind_is(true, "minecraft:third_person_front", AmmlCameraKind::THIRD_PERSON_FRONT);
}

#[test]
fn another_camera_is_other_on_the_70_26_layout() {
    kind_is(true, "minecraft:free", AmmlCameraKind::OTHER);
}

#[test]
fn the_registry_pointer_is_released_on_every_path() {
    let harness = start_in_world(true, &scene());
    let table = table(&harness);
    let before = ActiveCameraGame::uses();

    assert_eq!(active(&table, handle()).0, AmmlStatus::OK);
    assert_eq!(ActiveCameraGame::uses(), before);

    // No camera is active.
    ActiveCameraGame::build(true, &scene().map(|camera| SceneCamera { active: false, ..camera }));
    assert_eq!(active(&table, handle()).0, AmmlStatus::FAILED);
    assert_eq!(ActiveCameraGame::uses(), 100);

    // The registry is not valid any more.
    ActiveCameraGame::build(true, &scene());
    ActiveCameraGame::set_valid(false);
    assert_eq!(active(&table, handle()).0, AmmlStatus::FAILED);
    assert_eq!(ActiveCameraGame::uses(), 100);

    // The game gives an empty pointer.
    ActiveCameraGame::build(true, &scene());
    ActiveCameraGame::clear_registry();
    assert_eq!(active(&table, handle()).0, AmmlStatus::FAILED);
    assert_eq!(ActiveCameraGame::uses(), 100);
}

#[test]
fn an_entity_slot_that_is_not_engaged_is_skipped() {
    let mut cameras = scene();
    cameras[1].engaged = false;
    cameras[3].active = true;
    let harness = start_in_world(true, &cameras);

    assert_eq!(active(&table(&harness), handle()), (AmmlStatus::OK, AmmlCameraKind::OTHER));
}

#[test]
fn a_registry_without_cameras_gives_failed() {
    let harness = start_in_world(false, &[]);

    assert_eq!(active(&table(&harness), handle()).0, AmmlStatus::FAILED);
    assert_eq!(ActiveCameraGame::uses(), 100);
}

#[test]
fn a_client_that_is_not_the_primary_one_is_refused_without_a_game_call() {
    let harness = start_in_world(true, &scene());
    let table = table(&harness);
    let other = ClientGame::client(&GAME_CLIENT_B).cast_const();

    assert_eq!(active(&table, other).0, AmmlStatus::REFUSED);
    assert_eq!(ActiveCameraGame::uses(), 100);
}

#[test]
fn outside_a_world_the_call_is_refused() {
    let harness = install(true, None, &scene());

    assert_eq!(active(&table(&harness), handle()).0, AmmlStatus::REFUSED);
    assert_eq!(ActiveCameraGame::uses(), 100);
}

#[test]
fn a_null_out_is_invalid_argument() {
    let harness = start_in_world(true, &scene());
    let table = table(&harness);

    // SAFETY: the table is live; a null out is refused before it is written.
    let status = unsafe { (table.active_camera.unwrap())(handle(), std::ptr::null_mut()) };

    assert_eq!(status, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn an_empty_camera_vector_warns_nothing_and_a_bad_length_logs_one_warning() {
    let harness = start_in_world(true, &scene());
    let table = table(&harness);
    // A vector that never allocated: no camera and no warning, so the one warning of the process is still free.
    ActiveCameraGame::null_entities();
    assert_eq!(active(&table, handle()).0, AmmlStatus::FAILED);
    ActiveCameraGame::restore_entities();
    ActiveCameraGame::set_entities_bytes(0x30);

    assert_eq!(active(&table, handle()).0, AmmlStatus::FAILED);
    assert_eq!(active(&table, handle()).0, AmmlStatus::FAILED);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "bad length");
    assert_eq!(log.matches("camera layout mismatch").count(), 1, "{log}");
    assert!(log.contains("CameraRegistry.mCameraEntities"), "{log}");
    assert_eq!(ActiveCameraGame::uses(), 100);
}

#[test]
fn a_component_size_that_does_not_fit_the_layout_is_not_offered() {
    let harness = install(true, Some(0x120), &scene());
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);

    assert!(!harness.state.areas().is_available("camera.active", 1));
    assert_eq!(active(&table(&harness), handle()).0, AmmlStatus::NOT_AVAILABLE);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "camera layout mismatch");
    assert!(log.contains("camera layout mismatch"), "{log}");
}

#[test]
fn the_factor_shape_is_offered_on_the_26_63_layout() {
    let harness = install(false, None, &scene());

    assert!(harness.state.areas().is_available("camera.active", 1));
}

#[test]
fn the_size_shape_is_offered_on_the_70_26_layout() {
    let harness = install(true, None, &scene());

    assert!(harness.state.areas().is_available("camera.active", 1));
}

#[test]
fn a_factor_that_does_not_fit_the_layout_is_not_offered() {
    let harness = install(false, Some(10), &scene());

    assert!(!harness.state.areas().is_available("camera.active", 1));
}
