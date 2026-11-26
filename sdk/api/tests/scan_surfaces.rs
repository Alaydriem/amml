use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeChunk, FakeClient, FakeGame, FakeWorld};
use amml_api::{Area, BlockPos, BlockSource, ChunkLoad, ColumnPos, HookOrder, Screen, SurfaceSample};

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

const AREAS: [Area; 4] = [Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_CHUNK_READ, Area::WORLD_SCAN_SURFACES];

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

fn surface(y: i32, sky_light: u8, block_light: u8) -> SurfaceSample {
    SurfaceSample { pos: BlockPos { x: 5, y, z: 7 }, block_light, sky_light }
}

// A world with the chunk (0, 0) in state `load` and the surfaces `surfaces` in its column (5, 7), given in any order.
fn world_of(load: ChunkLoad, surfaces: Vec<SurfaceSample>) -> FakeWorld {
    let mut world = FakeWorld::default();
    world.chunks.insert((0, 0), FakeChunk::new(load));
    world.surfaces.insert((5, 7), surfaces);
    world
}

const COLUMN: ColumnPos = ColumnPos { x: 5, z: 7 };

#[test]
fn scan_of_an_unloaded_chunk_is_none() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(ChunkLoad::NotLoaded, vec![surface(10, 15, 0)])));

    let (not_loaded, no_chunk) = with_source(&fake, &AREAS, |source| {
        let mut out = [SurfaceSample::default(); 4];
        (
            source.scan_surfaces(COLUMN, -64, 320, &mut out),
            source.scan_surfaces(ColumnPos { x: 40, z: 7 }, -64, 320, &mut out),
        )
    });

    assert_eq!((not_loaded, no_chunk), (None, None));
}

#[test]
fn samples_come_lowest_first() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(
        ChunkLoad::Loaded,
        vec![surface(40, 15, 0), surface(10, 11, 4), surface(25, 5, 10)],
    )));

    let (count, out) = with_source(&fake, &AREAS, |source| {
        let mut out = [SurfaceSample::default(); 8];
        (source.scan_surfaces(COLUMN, -64, 320, &mut out), out)
    });

    assert_eq!(count, Some(3));
    assert_eq!(&out[..3], [surface(10, 11, 4), surface(25, 5, 10), surface(40, 15, 0)]);
}

#[test]
fn the_range_is_included_at_both_ends_and_the_room_cuts_the_list() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(
        ChunkLoad::Loaded,
        vec![surface(9, 0, 0), surface(10, 1, 1), surface(20, 2, 2), surface(30, 3, 3), surface(31, 4, 4)],
    )));

    let (ranged, short) = with_source(&fake, &AREAS, |source| {
        let mut out = [SurfaceSample::default(); 8];
        let ranged = source.scan_surfaces(COLUMN, 10, 30, &mut out).map(|count| out[..count].to_vec());
        let mut room = [SurfaceSample::default(); 2];
        let short = source.scan_surfaces(COLUMN, 10, 30, &mut room).map(|count| room[..count].to_vec());
        (ranged, short)
    });

    assert_eq!(ranged, Some(vec![surface(10, 1, 1), surface(20, 2, 2), surface(30, 3, 3)]));
    assert_eq!(short, Some(vec![surface(10, 1, 1), surface(20, 2, 2)]));
}

#[test]
fn a_column_without_surfaces_gives_zero() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(ChunkLoad::Loaded, Vec::new())));

    let count = with_source(&fake, &AREAS, |source| {
        let mut out = [SurfaceSample::default(); 4];
        source.scan_surfaces(COLUMN, -64, 320, &mut out)
    });

    assert_eq!(count, Some(0));
}

#[test]
fn the_scan_needs_its_area() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(ChunkLoad::Loaded, vec![surface(10, 15, 0)])));

    let none = with_source(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_CHUNK_READ], |source| {
        let mut out = [SurfaceSample::default(); 4];
        source.scan_surfaces(COLUMN, -64, 320, &mut out).is_none()
    });

    assert!(none);
}

#[test]
fn a_column_with_more_surfaces_than_one_batch_is_given_whole_and_in_order() {
    let fake = FakeGame::new();
    let surfaces: Vec<SurfaceSample> = (0..40).rev().map(|n| surface(n * 2, 15, 0)).collect();
    fake.set_client(in_world(world_of(ChunkLoad::Loaded, surfaces)));

    let (count, out) = with_source(&fake, &AREAS, |source| {
        let mut out = [SurfaceSample::default(); 64];
        (source.scan_surfaces(COLUMN, 0, 100, &mut out), out)
    });

    assert_eq!(count, Some(40));
    let expected: Vec<SurfaceSample> = (0..40).map(|n| surface(n * 2, 15, 0)).collect();
    assert_eq!(&out[..40], expected.as_slice());
}
