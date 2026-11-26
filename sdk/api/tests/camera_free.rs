use std::sync::{Arc, Mutex};

use amml_abi::{AmmlOrigin, CameraInstructionReceived, ClientTick};
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{
    ApiError, Area, CameraInstructionReceivedExt, CameraKind, Event, HookOrder, InstructionKind, Pose, Rotation,
    Screen, Vec3,
};

fn pose() -> Pose {
    Pose { position: Vec3 { x: 12.5, y: 70.0, z: -3.25 }, rotation: Rotation { yaw: 90.0, pitch: 10.0 }, roll: 0.0 }
}

fn offered() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<CameraInstructionReceived>();
    fake.offer_area(Area::CAMERA_FREE);
    fake.open_gate();
    fake.set_epoch(1);
    fake
}

// The kind and the origin of each `CameraInstructionReceived` call.
type Seen = Arc<Mutex<Vec<(InstructionKind, AmmlOrigin)>>>;

// A `CameraInstructionReceived` listener that records the kind and the origin of each call.
fn record(fake: &FakeGame) -> (Seen, amml_api::Listener) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let listener = fake
        .ctx()
        .events()
        .listen::<CameraInstructionReceived>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.kind(), event.header().origin));
        })
        .unwrap();
    (seen, listener)
}

#[test]
fn own_set_free_is_raised_with_origin_this_mod() {
    let fake = offered();
    let (seen, _listener) = record(&fake);
    fake.capture_handler();
    let camera = fake.ctx().camera();

    assert_eq!(camera.set_free(pose()), Ok(()));
    assert_eq!(camera.release_free(), Ok(()));

    assert_eq!(fake.camera_free(), ["set_free 12.5,70,-3.25 yaw=90 pitch=10", "release_free"]);
    assert_eq!(
        *seen.lock().unwrap(),
        [(InstructionKind::Set, AmmlOrigin::THIS_MOD), (InstructionKind::Clear, AmmlOrigin::THIS_MOD)]
    );
}

#[test]
fn a_server_instruction_is_raised_with_its_origin() {
    let fake = offered();
    let (seen, _listener) = record(&fake);

    fake.raise_camera_instruction(InstructionKind::Other, AmmlOrigin::GAME);

    assert_eq!(*seen.lock().unwrap(), [(InstructionKind::Other, AmmlOrigin::GAME)]);
}

#[test]
fn set_free_before_capture_is_not_ready() {
    let fake = offered();
    let camera = fake.ctx().camera();

    assert_eq!(camera.set_free(pose()), Err(ApiError::NotReady));
    assert_eq!(camera.release_free(), Err(ApiError::NotReady));
    assert!(fake.camera_free().is_empty());
}

#[test]
fn free_available_comes_from_the_fake() {
    let fake = offered();
    fake.capture_handler();
    let camera = fake.ctx().camera();
    assert_eq!(camera.free_available(), Some(true));

    fake.set_free_available(false);

    assert_eq!(camera.free_available(), Some(false));
    assert_eq!(camera.set_free(pose()), Err(ApiError::NotAvailable));
}

#[test]
fn without_the_area_the_free_camera_is_not_available() {
    let fake = FakeGame::new();
    fake.capture_handler();
    let camera = fake.ctx().camera();

    assert_eq!(camera.set_free(pose()), Err(ApiError::NotAvailable));
    assert_eq!(camera.free_available(), None);
    assert!(!fake.ctx().supports(Area::CAMERA_FREE));
}

fn in_world(active_camera: Option<CameraKind>) -> FakeClient {
    FakeClient {
        local_player: true,
        world_active: true,
        top_screen: Screen::Other,
        active_camera,
        ..FakeClient::default()
    }
}

// What `CameraControl::active_camera` gives in one `ClientTick`, after offering `areas`.
fn read_active(fake: &FakeGame, areas: &[Area]) -> Option<Option<CameraKind>> {
    fake.offer::<ClientTick>();
    for area in areas {
        fake.offer_area(*area);
    }
    fake.open_gate();
    let camera = fake.ctx().camera();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            *record.lock().unwrap() = Some(camera.active_camera(&at.client().unwrap()));
        })
        .unwrap();
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    seen.lock().unwrap().take()
}

#[test]
fn active_camera_comes_from_the_fake() {
    for kind in [CameraKind::FirstPerson, CameraKind::ThirdPerson, CameraKind::ThirdPersonFront, CameraKind::Other] {
        let fake = FakeGame::new();
        fake.set_client(in_world(Some(kind)));

        assert_eq!(read_active(&fake, &[Area::CAMERA_ACTIVE]), Some(Some(kind)));
    }
}

#[test]
fn active_camera_is_none_without_the_area() {
    let fake = FakeGame::new();
    fake.set_client(in_world(Some(CameraKind::FirstPerson)));

    assert_eq!(read_active(&fake, &[]), Some(None));
}

#[test]
fn active_camera_is_none_when_the_game_has_no_active_camera() {
    let fake = FakeGame::new();
    fake.set_client(in_world(None));

    assert_eq!(read_active(&fake, &[Area::CAMERA_ACTIVE]), Some(None));
}
