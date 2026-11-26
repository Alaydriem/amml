use std::sync::{Arc, Mutex};

use amml_abi::{AmmlPhase, AmmlStatus, CameraSetup};
use amml_api::testing::{FakeCamera, FakeGame};
use amml_api::{ApiError, Area, CameraSetupExt, Event, HookOrder, Pose, Projection, Rotation, Vec3};

fn pose() -> Pose {
    Pose { position: Vec3 { x: 12.5, y: 70.0, z: -3.25 }, rotation: Rotation { yaw: 90.0, pitch: 10.0 }, roll: 5.0 }
}

fn projection() -> Projection {
    Projection { fov_y: 70.0, aspect: 16.0 / 9.0 }
}

fn offered() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<CameraSetup>();
    fake.offer_area(Area::CAMERA_POSE);
    fake.offer_area(Area::CAMERA_PROJECTION);
    fake
}

#[test]
fn camera_setup_carries_the_client_once_set() {
    let fake = offered();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<CameraSetup>(HookOrder::NORMAL, move |_, at| record.lock().unwrap().push(at.client().is_some()))
        .unwrap();

    fake.raise_camera_setup(AmmlPhase::BEFORE, &FakeCamera::default());
    fake.set_client(amml_api::testing::FakeClient::default());
    fake.raise_camera_setup(AmmlPhase::BEFORE, &FakeCamera::default());

    assert_eq!(*seen.lock().unwrap(), vec![false, true]);
}

#[test]
fn camera_is_none_for_a_copy_of_the_event() {
    let fake = offered();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let kept = Arc::new(Mutex::new(None));
    let keep = kept.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<CameraSetup>(HookOrder::NORMAL, move |event, _| {
            let copy = *event;
            record.lock().unwrap().push((event.camera().is_some(), copy.camera().is_some()));
            *keep.lock().unwrap() = Some(copy);
        })
        .unwrap();

    fake.raise_camera_setup(AmmlPhase::BEFORE, &FakeCamera::default());

    assert_eq!(*seen.lock().unwrap(), vec![(true, false)]);
    assert!(kept.lock().unwrap().unwrap().camera().is_none());
}

#[test]
fn pose_is_recorded() {
    let fake = offered();
    let results = Arc::new(Mutex::new(Vec::new()));
    let record = results.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<CameraSetup>(HookOrder::NORMAL, move |event, _| {
            let Some(camera) = event.camera() else { return };
            let bad = Pose { roll: f32::NAN, ..pose() };
            record.lock().unwrap().push((
                event.frame_id(),
                event.first_in_frame(),
                camera.set_pose(pose()),
                camera.set_pose(bad),
            ));
        })
        .unwrap();
    let camera = FakeCamera::default();

    fake.raise_camera_setup(AmmlPhase::BEFORE, &camera);

    let invalid = Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT));
    assert_eq!(*results.lock().unwrap(), vec![(1, true, Ok(()), invalid)]);
    assert_eq!(camera.poses(), vec![pose()]);
}

#[test]
fn projection_comes_from_the_fake() {
    let fake = offered();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<CameraSetup>(HookOrder::NORMAL, move |event, _| {
            let Some(camera) = event.camera() else { return };
            let after = event.header().phase == AmmlPhase::AFTER;
            record.lock().unwrap().push((after, camera.projection(), camera.rendered_position()));
        })
        .unwrap();
    let rendered = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    let camera = FakeCamera { projection: Some(projection()), rendered: Some(rendered), ..Default::default() };

    fake.raise_camera_setup(AmmlPhase::BEFORE, &camera);
    fake.raise_camera_setup(AmmlPhase::AFTER, &camera);

    // The projection is read after the game's setup only.
    assert_eq!(*seen.lock().unwrap(), vec![(false, None, Some(rendered)), (true, Some(projection()), Some(rendered))]);
}

#[test]
fn after_shares_the_frame_of_its_before() {
    let fake = offered();
    let frames = Arc::new(Mutex::new(Vec::new()));
    let record = frames.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<CameraSetup>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(event.frame_id());
        })
        .unwrap();

    fake.raise_camera_setup(AmmlPhase::BEFORE, &FakeCamera::default());
    fake.raise_camera_setup(AmmlPhase::AFTER, &FakeCamera::default());
    fake.raise_camera_setup(AmmlPhase::BEFORE, &FakeCamera::default());

    assert_eq!(*frames.lock().unwrap(), vec![1, 1, 2]);
}

#[test]
fn camera_calls_without_the_areas_are_not_available() {
    let fake = FakeGame::new();
    fake.offer::<CameraSetup>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<CameraSetup>(HookOrder::NORMAL, move |event, _| {
            let Some(camera) = event.camera() else { return };
            record.lock().unwrap().push((camera.set_pose(pose()), camera.projection()));
        })
        .unwrap();
    let camera = FakeCamera { projection: Some(projection()), ..Default::default() };

    fake.raise_camera_setup(AmmlPhase::AFTER, &camera);

    assert_eq!(*seen.lock().unwrap(), vec![(Err(ApiError::NotAvailable), None)]);
    assert!(camera.poses().is_empty());
}

#[test]
fn a_loader_without_a_camera_table_gives_no_camera() {
    let fake = FakeGame::with_v10_services();
    fake.offer::<CameraSetup>();
    fake.offer_area(Area::CAMERA_POSE);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<CameraSetup>(HookOrder::NORMAL, move |event, _| record.lock().unwrap().push(event.camera().is_some()))
        .unwrap();

    fake.raise_camera_setup(AmmlPhase::BEFORE, &FakeCamera::default());

    assert_eq!(*seen.lock().unwrap(), vec![false]);
    assert!(!fake.ctx().supports(Area::CAMERA_POSE));
}
