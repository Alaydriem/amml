use std::sync::{Arc, Mutex};

use amml_abi::{AmmlGameType, AmmlPermissionLevel, ClientTick};
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{Aabb, ApiError, Area, GameType, HookOrder, PermissionLevel, Rotation, Screen, Vec3};

fn tick() -> ClientTick {
    ClientTick { header: FakeGame::header::<ClientTick>() }
}

fn in_world() -> FakeClient {
    FakeClient { local_player: true, world_active: true, top_screen: Screen::Other, ..FakeClient::default() }
}

// Runs `body` with the local player of one `ClientTick` after offering `areas`; gives what `body` gave.
fn with_player<R: Send + 'static>(
    fake: &FakeGame,
    areas: &[Area],
    body: impl Fn(&amml_api::LocalPlayer<'_>) -> R + Send + 'static,
) -> R {
    fake.offer::<ClientTick>();
    for area in areas {
        fake.offer_area(*area);
    }
    fake.open_gate();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let client = at.client().unwrap();
            *record.lock().unwrap() = Some(body(&client.local_player().unwrap()));
        })
        .unwrap();
    fake.raise_with_client(&mut tick());
    let taken = seen.lock().unwrap().take();
    taken.unwrap()
}

#[test]
fn game_type_and_permission_from_the_fake() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient {
        game_type: Some(GameType::Creative),
        permission_level: Some(PermissionLevel::Operator),
        ..in_world()
    });

    let reads = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_GAME_TYPE, Area::ACTOR_PERMISSION], |player| {
        (player.game_type(), player.permission_level())
    });

    assert_eq!(reads, (Some(GameType::Creative), Some(PermissionLevel::Operator)));
}

#[test]
fn game_type_and_permission_need_their_own_areas() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient {
        game_type: Some(GameType::Survival),
        permission_level: Some(PermissionLevel::Member),
        ..in_world()
    });

    let reads = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(4)], |player| {
        (player.game_type(), player.permission_level())
    });

    assert_eq!(reads, (None, None));
}

#[test]
fn unknown_game_type_reads_other() {
    assert_eq!(GameType::from(AmmlGameType(3)), GameType::Other(3));
    assert_eq!(GameType::from(AmmlGameType(-1)), GameType::Other(-1));
    assert_eq!(GameType::from(AmmlGameType(5)), GameType::Default);
    assert_eq!(GameType::from(AmmlGameType(6)), GameType::Spectator);
    assert_eq!(AmmlGameType::from(GameType::Other(42)), AmmlGameType(42));

    let fake = FakeGame::new();
    fake.set_client(FakeClient { game_type: Some(GameType::Other(42)), ..in_world() });
    let read = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_GAME_TYPE], |player| player.game_type());
    assert_eq!(read, Some(GameType::Other(42)));
}

#[test]
fn operator_orders_above_member() {
    assert!(PermissionLevel::Operator > PermissionLevel::Member);
    assert!(PermissionLevel::Member > PermissionLevel::Visitor);
    assert!(PermissionLevel::Custom > PermissionLevel::Operator);
    assert!(PermissionLevel::Other(9) > PermissionLevel::Custom);
    assert_eq!(PermissionLevel::from(AmmlPermissionLevel(2)), PermissionLevel::Operator);
    assert_eq!(PermissionLevel::from(AmmlPermissionLevel(7)), PermissionLevel::Other(7));
    // A raw value the enum names is never an `Other`: the two compare equal.
    assert_eq!(PermissionLevel::Other(1), PermissionLevel::Member);
}

#[test]
fn view_vector_and_aabb_come_from_the_fake() {
    let fake = FakeGame::new();
    let aabb = Aabb { min: Vec3 { x: 1.0, y: 64.0, z: 2.0 }, max: Vec3 { x: 1.6, y: 65.8, z: 2.6 } };
    fake.set_client(FakeClient { view_vector: Some(Vec3 { x: 0.0, y: -0.5, z: 0.8 }), aabb: Some(aabb), ..in_world() });

    let reads = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(4)], |player| {
        (player.view_vector(), player.aabb())
    });

    assert_eq!(reads, (Some(Vec3 { x: 0.0, y: -0.5, z: 0.8 }), Some(aabb)));
}

#[test]
fn view_vector_needs_read_v4() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient {
        view_vector: Some(Vec3 { x: 0.0, y: 0.0, z: 1.0 }),
        aabb: Some(Aabb::default()),
        ..in_world()
    });

    let reads = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(3)], |player| {
        (player.view_vector(), player.aabb())
    });

    assert_eq!(reads, (None, None));
}

#[test]
fn set_rotation_is_recorded_and_changes_the_rotation_read() {
    let fake = FakeGame::new();
    fake.set_client(in_world());

    let results = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(2), Area::ACTOR_LOOK_WRITE], |player| {
        let first = player.set_rotation(Rotation { yaw: 10.0, pitch: -5.0 }, None);
        let second = player.set_rotation(Rotation { yaw: 20.0, pitch: 3.0 }, Some(18.0));
        (first, second, player.rotation())
    });

    assert_eq!(results, (Ok(()), Ok(()), Some(Rotation { yaw: 20.0, pitch: 3.0 })));
    assert_eq!(
        fake.rotation_writes(),
        vec![(Rotation { yaw: 10.0, pitch: -5.0 }, None), (Rotation { yaw: 20.0, pitch: 3.0 }, Some(18.0))]
    );
}

#[test]
fn set_rotation_rejects_a_number_that_is_not_finite() {
    let fake = FakeGame::new();
    fake.set_client(in_world());

    let results = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_LOOK_WRITE], |player| {
        [
            player.set_rotation(Rotation { yaw: f32::NAN, pitch: 0.0 }, None),
            player.set_rotation(Rotation { yaw: 0.0, pitch: 0.0 }, Some(f32::INFINITY)),
        ]
    });

    let invalid = Err(ApiError::Loader(amml_abi::AmmlStatus::INVALID_ARGUMENT));
    assert_eq!(results, [invalid, invalid]);
    assert!(fake.rotation_writes().is_empty());
}

#[test]
fn set_rotation_without_its_area_is_not_available() {
    let fake = FakeGame::new();
    fake.set_client(in_world());

    let result =
        with_player(&fake, &[Area::CLIENT_READ], |player| player.set_rotation(Rotation { yaw: 1.0, pitch: 1.0 }, None));

    assert_eq!(result, Err(ApiError::NotAvailable));
    assert!(fake.rotation_writes().is_empty());
}
