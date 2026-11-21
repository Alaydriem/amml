use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeClient, FakeGame, FakeWorld};
use amml_api::{Aabb, ActorBox, Area, BlockHit, BlockPos, BlockSource, Face, HookOrder, RayOptions, Screen, Vec3};

fn tick() -> ClientTick {
    ClientTick { header: FakeGame::header::<ClientTick>() }
}

fn in_world(world: FakeWorld) -> FakeClient {
    FakeClient {
        local_player: true,
        world_active: true,
        top_screen: Screen::Other,
        world: Some(world),
        ..FakeClient::default()
    }
}

const AREAS: [Area; 4] = [Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_RAYCAST, Area::WORLD_ACTORS_IN];

// Runs `body` with the block source of the local player of one `ClientTick`, after offering `areas`.
fn with_source<R: Send + 'static>(
    fake: &FakeGame,
    areas: &[Area],
    body: impl Fn(&BlockSource<'_>) -> R + Send + 'static,
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
            let source = client.local_player().unwrap().block_source().unwrap();
            *record.lock().unwrap() = Some(body(&source));
        })
        .unwrap();
    fake.raise_with_client(&mut tick());
    let taken = seen.lock().unwrap().take();
    taken.unwrap()
}

fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    Aabb { min, max }
}

const LOOK: RayOptions = RayOptions { every_block: true, outline: true, max_steps: 0 };

#[test]
fn raycast_hit_comes_from_the_fake() {
    let fake = FakeGame::new();
    let hit = BlockHit { pos: BlockPos { x: 3, y: 60, z: -2 }, point: vec3(3.5, 61.0, -1.5), face: Face::Up };
    fake.set_client(in_world(FakeWorld { ray_hit: Some(hit), ..FakeWorld::default() }));

    let (found, not_finite) = with_source(&fake, &AREAS, |source| {
        (
            source.raycast(vec3(3.5, 64.0, -1.5), vec3(3.5, 50.0, -1.5), LOOK),
            source.raycast(vec3(f32::NAN, 64.0, -1.5), vec3(3.5, 50.0, -1.5), LOOK),
        )
    });

    assert_eq!(found, Some(hit));
    assert_eq!(not_finite, None);
}

#[test]
fn raycast_of_a_world_with_no_hit_is_none() {
    let fake = FakeGame::new();
    fake.set_client(in_world(FakeWorld::default()));

    let miss = with_source(&fake, &AREAS, |source| source.raycast(vec3(0.0, 64.0, 0.0), vec3(0.0, 50.0, 0.0), LOOK));

    assert_eq!(miss, None);
}

#[test]
fn raycast_without_the_area_is_none() {
    let fake = FakeGame::new();
    let hit = BlockHit { pos: BlockPos::default(), point: vec3(0.5, 1.0, 0.5), face: Face::Up };
    fake.set_client(in_world(FakeWorld { ray_hit: Some(hit), ..FakeWorld::default() }));

    let without =
        with_source(&fake, &AREAS[..2], |source| source.raycast(vec3(0.0, 64.0, 0.0), vec3(0.0, 50.0, 0.0), LOOK));

    assert_eq!(without, None);
}

#[test]
fn actors_in_marks_the_local_player() {
    let fake = FakeGame::new();
    let mob = ActorBox { aabb: aabb(vec3(1.0, 60.0, 1.0), vec3(1.6, 61.8, 1.6)), is_local_player: false };
    let player = ActorBox { aabb: aabb(vec3(0.0, 64.0, 0.0), vec3(0.6, 65.8, 0.6)), is_local_player: true };
    let far = ActorBox { aabb: aabb(vec3(90.0, 60.0, 90.0), vec3(90.6, 61.8, 90.6)), is_local_player: false };
    fake.set_client(in_world(FakeWorld { actors: vec![mob, player, far], ..FakeWorld::default() }));

    let found =
        with_source(&fake, &AREAS, |source| source.actors_in(aabb(vec3(-8.0, 56.0, -8.0), vec3(8.0, 72.0, 8.0))));

    assert_eq!(found, vec![mob, player]);
}

#[test]
fn actors_in_gives_at_most_256_boxes() {
    let fake = FakeGame::new();
    let many: Vec<ActorBox> = (0..300)
        .map(|index| {
            let x = index as f32 * 0.01;
            ActorBox { aabb: aabb(vec3(x, 60.0, 0.0), vec3(x + 0.6, 61.8, 0.6)), is_local_player: false }
        })
        .collect();
    fake.set_client(in_world(FakeWorld { actors: many, ..FakeWorld::default() }));

    let found =
        with_source(&fake, &AREAS, |source| source.actors_in(aabb(vec3(-8.0, 56.0, -8.0), vec3(8.0, 72.0, 8.0))));

    assert_eq!(found.len(), 256);
}
