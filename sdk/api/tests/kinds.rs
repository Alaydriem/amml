use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeBlock, FakeChunk, FakeClient, FakeGame, FakeWorld};
use amml_api::{ApiError, Area, BlockPos, BlockSource, BlockTypeId, ChunkLoad, HookOrder, KindsInBox, Screen};

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

const AIR: BlockTypeId = BlockTypeId(1);
const STONE: BlockTypeId = BlockTypeId(2);
const DIRT: BlockTypeId = BlockTypeId(3);
const AREAS: [Area; 4] = [Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_CHUNK_READ, Area::WORLD_KINDS_IN_BOX];

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

fn block(type_id: BlockTypeId, name: &'static str) -> FakeBlock {
    FakeBlock { type_id, name, is_air: type_id == AIR, ..FakeBlock::default() }
}

// Chunk (0, 0) loaded, the three names, stone at (0, 1, 0) and dirt at (1, 0, 0); every other cell is air.
fn world() -> FakeWorld {
    let mut world = FakeWorld::default();
    world.chunks.insert((0, 0), FakeChunk::new(ChunkLoad::Loaded));
    world.names.insert(AIR, "minecraft:air".to_string());
    world.names.insert(STONE, "minecraft:stone".to_string());
    world.names.insert(DIRT, "minecraft:dirt".to_string());
    world.blocks.insert(pos(0, 1, 0), block(STONE, "minecraft:stone"));
    world.blocks.insert(pos(1, 0, 0), block(DIRT, "minecraft:dirt"));
    world
}

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
    fake.set_epoch(1);
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

#[test]
fn kinds_are_named_through_the_name_table() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world()));

    // x, then z, then y: air at (0, 0, 0), stone at (0, 1, 0), air at (0, 2, 0), dirt at (1, 0, 0).
    let found = with_source(&fake, &AREAS, |source| source.kinds_in_box(pos(0, 0, 0), pos(1, 2, 0), 8));

    assert_eq!(
        found,
        Ok(KindsInBox::Kinds(vec![
            "minecraft:air".to_string(),
            "minecraft:stone".to_string(),
            "minecraft:dirt".to_string()
        ]))
    );
}

#[test]
fn a_chunk_that_is_not_loaded_gives_not_loaded() {
    let fake = FakeGame::new();
    let mut world = world();
    world.chunks.insert((1, 0), FakeChunk::new(ChunkLoad::NotLoaded));
    fake.set_client(in_world(world));

    let (partly, outside) = with_source(&fake, &AREAS, |source| {
        (source.kinds_in_box(pos(0, 0, 0), pos(20, 3, 3), 8), source.kinds_in_box(pos(0, 0, 0), pos(40, 3, 3), 8))
    });

    assert_eq!((partly, outside), (Ok(KindsInBox::NotLoaded), Ok(KindsInBox::NotLoaded)));
}

#[test]
fn more_kinds_than_max_kinds_gives_too_many() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world()));

    let (two, three) = with_source(&fake, &AREAS, |source| {
        (source.kinds_in_box(pos(0, 0, 0), pos(1, 2, 0), 2), source.kinds_in_box(pos(0, 0, 0), pos(1, 2, 0), 3))
    });

    assert_eq!(two, Ok(KindsInBox::TooMany));
    assert!(matches!(three, Ok(KindsInBox::Kinds(kinds)) if kinds.len() == 3));
}

#[test]
fn an_inverted_box_and_a_box_over_the_cap_are_refused() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world()));

    let (inverted, over, exact) = with_source(&fake, &AREAS, |source| {
        (
            source.kinds_in_box(pos(2, 0, 0), pos(1, 0, 0), 8),
            source.kinds_in_box(pos(0, 0, 0), pos(15, 256, 15), 8),
            source.kinds_in_box(pos(0, 0, 0), pos(15, 255, 15), 8),
        )
    });

    assert_eq!((inverted, over), (Err(ApiError::Refused), Err(ApiError::Refused)));
    assert!(matches!(exact, Ok(KindsInBox::Kinds(_))));
    assert_eq!(BlockSource::KINDS_CELL_CAP, 65536);
}

#[test]
fn without_the_area_the_call_is_not_available() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world()));

    let found = with_source(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_CHUNK_READ], |source| {
        source.kinds_in_box(pos(0, 0, 0), pos(1, 1, 1), 8)
    });

    assert_eq!(found, Err(ApiError::NotAvailable));
}
