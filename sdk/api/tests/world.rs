use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeClient, FakeGame, FakeWorld};
use amml_api::{Area, BlockSource, HookOrder, Screen, Vec3};

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
fn head_position_and_health_come_from_the_fake() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient {
        head_position: Some(Vec3 { x: 1.0, y: 66.0, z: 2.0 }),
        health: Some(14),
        ..in_world()
    });

    let reads = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::ACTOR_HEALTH], |player| {
        (player.head_position(), player.health())
    });

    assert_eq!(reads, (Some(Vec3 { x: 1.0, y: 66.0, z: 2.0 }), Some(14)));
}

#[test]
fn head_position_needs_actor_read_version_3_and_health_its_own_area() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient { head_position: Some(Vec3::default()), health: Some(20), ..in_world() });

    let reads = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(2)], |player| {
        (player.head_position(), player.health())
    });

    assert_eq!(reads, (None, None));
}

#[test]
fn block_source_is_none_without_a_world() {
    let fake = FakeGame::new();
    fake.set_client(in_world());

    let source =
        with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(3)], |player| player.block_source().is_some());

    assert!(!source);
}

#[test]
fn block_source_is_some_with_a_world_and_equal_to_itself() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient { world: Some(FakeWorld::default()), ..in_world() });

    let equal = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(3)], |player| {
        let first: Option<BlockSource<'_>> = player.block_source();
        let second = player.block_source();
        first.is_some() && first == second
    });

    assert!(equal);
}

#[test]
fn sky_darken_needs_its_area() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient { dimension: Some(0), sky_darken: Some(9), ..in_world() });

    let without = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(2)], |player| {
        player.dimension().unwrap().sky_darken()
    });
    let with = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(2), Area::WORLD_SKY_DARKEN], |player| {
        player.dimension().unwrap().sky_darken()
    });

    assert_eq!((without, with), (None, Some(9)));
}

#[test]
fn a_loader_before_the_world_table_serves_no_world_area() {
    let fake = FakeGame::with_v8_services();
    fake.set_client(FakeClient { dimension: Some(0), sky_darken: Some(9), ..in_world() });

    let read = with_player(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(2), Area::WORLD_SKY_DARKEN], |player| {
        player.dimension().unwrap().sky_darken()
    });

    assert_eq!(read, None);
    assert!(!fake.ctx().supports(Area::WORLD_SKY_DARKEN));
}
