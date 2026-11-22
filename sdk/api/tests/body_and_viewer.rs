use std::sync::{Arc, Mutex};

use amml_abi::PlayerMoved;
use amml_api::testing::FakeGame;
use amml_api::{ActorTarget, ApiError, Area, HookOrder, PlayerMovedExt, Rotation, Vec3};

fn offered() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer_area(Area::CAMERA_HIDE_ACTOR);
    fake.open_gate();
    fake.set_epoch(1);
    fake
}

#[test]
fn hide_actor_needs_its_area() {
    let fake = FakeGame::new();
    fake.open_gate();
    fake.set_epoch(1);

    assert_eq!(fake.ctx().camera().hide_actor(ActorTarget::LocalPlayer, true), Err(ApiError::NotAvailable));
    assert!(!fake.body_hidden());
    assert!(!fake.ctx().supports(Area::CAMERA_HIDE_ACTOR));

    fake.offer_area(Area::CAMERA_HIDE_ACTOR);
    assert!(fake.ctx().supports(Area::CAMERA_HIDE_ACTOR));
    assert_eq!(fake.ctx().camera().hide_actor(ActorTarget::LocalPlayer, true), Ok(()));
    assert!(fake.body_hidden());

    assert_eq!(fake.ctx().camera().hide_actor(ActorTarget::LocalPlayer, false), Ok(()));
    assert!(!fake.body_hidden());
}

#[test]
fn hide_actor_ends_with_the_world() {
    let fake = offered();
    fake.ctx().camera().hide_actor(ActorTarget::LocalPlayer, true).unwrap();
    assert!(fake.body_hidden());

    fake.leave_camera_world();

    assert!(!fake.body_hidden());
}

#[test]
fn hide_actor_without_a_camera_table_is_not_available() {
    let fake = FakeGame::with_v12_services();
    fake.open_gate();

    assert_eq!(fake.ctx().camera().hide_actor(ActorTarget::LocalPlayer, true), Err(ApiError::NotAvailable));
}

#[test]
fn player_moved_from_the_fake() {
    let fake = offered();
    fake.offer::<PlayerMoved>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PlayerMoved>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.pos(), event.rotation(), event.head_yaw(), event.mode()));
        })
        .unwrap();

    fake.raise_player_moved(Vec3 { x: 10.5, y: 64.0, z: -3.25 }, Rotation { yaw: 90.0, pitch: 15.0 }, 45.0, 2);

    assert_eq!(
        *seen.lock().unwrap(),
        [(Vec3 { x: 10.5, y: 64.0, z: -3.25 }, Rotation { yaw: 90.0, pitch: 15.0 }, 45.0, 2)]
    );
}
