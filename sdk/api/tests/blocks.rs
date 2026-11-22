use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeBlock, FakeClient, FakeGame, FakeItem, FakeWorld};
use amml_api::{Area, Block, BlockHandle, BlockPos, BlockSource, BlockTypeId, HookOrder, ItemSnapshot, Screen};

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

const STAIRS: BlockPos = BlockPos { x: 1, y: 64, z: 2 };
const AIR: BlockPos = BlockPos { x: 1, y: 65, z: 2 };
const AREAS: [Area; 4] = [Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_CHUNK_READ, Area::WORLD_BLOCKS];

// What the reads of one `Block` gave.
#[derive(Debug, PartialEq)]
struct Reads {
    type_id: Option<BlockTypeId>,
    name: Option<String>,
    is_air: Option<bool>,
    built_over: Option<bool>,
    interactive: Option<bool>,
    item: Option<ItemSnapshot>,
    handle: Option<BlockHandle>,
}

impl Reads {
    fn of(block: &Block<'_>) -> Self {
        Self {
            type_id: block.type_id(),
            name: block.name(),
            is_air: block.is_air(),
            built_over: block.can_be_built_over(),
            interactive: block.is_interactive(),
            item: block.item(),
            handle: block.handle(),
        }
    }
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

fn world() -> FakeWorld {
    let mut world = FakeWorld::default();
    world.blocks.insert(
        STAIRS,
        FakeBlock {
            type_id: BlockTypeId(7),
            name: "minecraft:oak_stairs",
            is_air: false,
            built_over: false,
            interactive: true,
            item: Some(FakeItem { type_name: "minecraft:oak_stairs", count: 1, ..FakeItem::EMPTY }),
            handle: Some(BlockHandle::for_test(9)),
        },
    );
    world.blocks.insert(
        AIR,
        FakeBlock {
            type_id: BlockTypeId(1),
            name: "minecraft:air",
            is_air: true,
            built_over: true,
            interactive: false,
            item: None,
            handle: Some(BlockHandle::for_test(10)),
        },
    );
    world
}

#[test]
fn block_reads_from_the_fake() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world()));

    let (stairs, air, missing) = with_source(&fake, &AREAS, |source| {
        (
            source.block(STAIRS).map(|block| Reads::of(&block)),
            source.block(AIR).map(|block| Reads::of(&block)),
            source.block(BlockPos { x: 0, y: 0, z: 0 }).map(|block| (block.is_air(), block.item())),
        )
    });

    assert_eq!(
        stairs,
        Some(Reads {
            type_id: Some(BlockTypeId(7)),
            name: Some("minecraft:oak_stairs".to_string()),
            is_air: Some(false),
            built_over: Some(false),
            interactive: Some(true),
            item: Some(ItemSnapshot {
                type_name: Some("minecraft:oak_stairs".to_string()),
                count: 1,
                damage: Some(0),
                max_damage: Some(0),
                ..ItemSnapshot::default()
            }),
            handle: Some(BlockHandle::for_test(9)),
        })
    );
    let air = air.unwrap();
    assert_eq!((air.is_air, air.built_over, air.interactive), (Some(true), Some(true), Some(false)));
    assert_eq!(air.item, None);
    // A position the fake world does not map reads as air, as the game gives air for any position.
    assert_eq!(missing, Some((Some(true), None)));
}

#[test]
fn block_needs_its_area() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world()));

    let (offered, withdrawn) = {
        let offered = with_source(&fake, &AREAS, |source| source.block(STAIRS).is_some());
        fake.withdraw_area(Area::WORLD_BLOCKS);
        let withdrawn = with_source(&fake, &[], |source| source.block(STAIRS).is_some());
        (offered, withdrawn)
    };

    assert!(offered);
    assert!(!withdrawn);
    assert!(!fake.ctx().supports(Area::WORLD_BLOCKS));
}

#[test]
fn unmapped_air_can_be_built_over() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world()));

    // A position the world does not map reads as the fake's air, which a block can be placed into, as in the game.
    let built_over = with_source(&fake, &AREAS, |source| {
        source.block(BlockPos { x: 0, y: 0, z: 0 }).map(|block| (block.is_air(), block.can_be_built_over()))
    });

    assert_eq!(built_over, Some((Some(true), Some(true))));
}
