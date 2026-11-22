use amml_api::testing::{FakeClient, FakeGame, FakeItem, FakeWorld};
use amml_api::{ApiError, Area, BlockHandle, BlockMatch, Screen, StateValue};

const STAIRS: &str = "minecraft:oak_stairs";

fn in_world(world: FakeWorld) -> FakeClient {
    FakeClient {
        local_player: true,
        world_active: true,
        top_screen: Screen::Other,
        world: Some(world),
        ..FakeClient::default()
    }
}

fn joined(world: FakeWorld) -> FakeGame {
    let fake = FakeGame::new();
    fake.set_client(in_world(world));
    fake.offer_area(Area::WORLD_BLOCK_STATES);
    fake.open_gate();
    fake.set_epoch(1);
    fake
}

fn state(name: &str, value: &str) -> (String, String) {
    (name.to_string(), value.to_string())
}

#[test]
fn from_state_exact_from_the_fake() {
    let (upside_down, default) = (BlockHandle::for_test(7), BlockHandle::for_test(8));
    let mut world = FakeWorld::default();
    world.states.insert(
        (STAIRS.to_string(), vec![state("upside_down_bit", "true"), state("weirdo_direction", "2")]),
        upside_down,
    );
    world.default_states.insert(STAIRS.to_string(), default);
    let fake = joined(world);
    let blocks = fake.ctx().blocks();

    let exact = blocks
        .from_state(STAIRS, &[("weirdo_direction", StateValue::Int(2)), ("upside_down_bit", StateValue::Bool(true))]);
    let unknown = blocks.from_state(STAIRS, &[("facing", StateValue::Text("north"))]);
    let missing = blocks.from_state("minecraft:no_such_block", &[]);

    assert_eq!(exact, Some(BlockMatch { block: upside_down, exact: true }));
    assert_eq!(unknown, Some(BlockMatch { block: default, exact: false }));
    assert_eq!(missing, None);
}

#[test]
fn from_state_needs_the_area_and_a_world() {
    let mut world = FakeWorld::default();
    world.default_states.insert(STAIRS.to_string(), BlockHandle::for_test(8));
    let fake = joined(world);
    fake.withdraw_area(Area::WORLD_BLOCK_STATES);

    assert_eq!(fake.ctx().blocks().from_state(STAIRS, &[]), None);
    assert!(!fake.ctx().supports(Area::WORLD_BLOCK_STATES));
}

#[test]
fn rotated_records_the_quarters_and_gives_the_turned_block() {
    let (east, south) = (BlockHandle::for_test(7), BlockHandle::for_test(8));
    let mut world = FakeWorld::default();
    world.rotated.insert((east, 1), south);
    let fake = joined(world);
    let blocks = fake.ctx().blocks();

    assert_eq!(blocks.rotated(east, 5), Ok(south));
    assert_eq!(blocks.rotated(south, 2), Ok(south));
    assert_eq!(fake.rotations(), vec![(east, 1), (south, 2)]);
    fake.set_epoch(0);
    assert_eq!(blocks.rotated(east, 1), Err(ApiError::NotReady));
}

#[test]
fn name_and_item_of_a_handle() {
    let block = BlockHandle::for_test(7);
    let mut world = FakeWorld::default();
    world.block_names.insert(block, STAIRS.to_string());
    world.block_items.insert(block, FakeItem { type_name: STAIRS, count: 1, max_count: Some(64), ..FakeItem::EMPTY });
    let fake = joined(world);
    let blocks = fake.ctx().blocks();

    let item = blocks.item(block).unwrap();

    assert_eq!(blocks.name(block).as_deref(), Some(STAIRS));
    assert_eq!((item.type_name.as_deref(), item.count, item.max_count), (Some(STAIRS), 1, Some(64)));
    assert_eq!(blocks.name(BlockHandle::for_test(9)), None);
    assert_eq!(blocks.item(BlockHandle::for_test(9)), None);
}
