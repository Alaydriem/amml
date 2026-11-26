use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeClient, FakeGame, FakeInventory, FakeItem};
use amml_api::{ApiError, Area, BlockHandle, Hand, HookOrder, LightFalloff, LightSource, Screen, Vec3};

const TORCH: FakeItem =
    FakeItem { type_name: "minecraft:torch", count: 1, damage: 0, max_damage: 0, ..FakeItem::EMPTY };
const STICK: FakeItem =
    FakeItem { type_name: "minecraft:stick", count: 1, damage: 0, max_damage: 0, ..FakeItem::EMPTY };

fn serving() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_READ);
    fake.offer_area(Area::INVENTORY_READ.at(2));
    fake.offer_area(Area::LIGHTING_TERRAIN);
    fake.open_gate();
    fake.set_epoch(1);
    fake.set_client(FakeClient {
        local_player: true,
        world_active: true,
        top_screen: Screen::Other,
        ..FakeClient::default()
    });
    fake.set_item_block("minecraft:torch", 14);
    fake
}

// Per hand: the light emission and the block handle the held item gives in one tick.
fn hands_in_tick(fake: &FakeGame) -> [Option<(Option<u8>, Option<BlockHandle>)>; 2] {
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let player = at.client().and_then(|client| client.local_player());
            let read = |hand| {
                player
                    .as_ref()
                    .and_then(|player| player.held_item(hand))
                    .map(|item| (item.light_emission(), item.block()))
            };
            *record.lock().unwrap() = Some([read(Hand::Main), read(Hand::Off)]);
        })
        .unwrap();
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    seen.lock().unwrap().take().unwrap()
}

fn light(block: Option<BlockHandle>) -> LightSource {
    LightSource { position: Vec3 { x: 0.5, y: 64.0, z: 0.5 }, level: 14, falloff: LightFalloff::Round, block }
}

#[test]
fn a_held_torch_gives_its_emission_and_a_block_in_either_hand() {
    let fake = serving();
    fake.set_inventory(FakeInventory { held: Some(STICK), selected_slot: Some(0), ..FakeInventory::default() });
    fake.set_off_hand(Some(TORCH));

    let [main, off] = hands_in_tick(&fake);

    assert_eq!(main, Some((None, None)));
    let (emission, block) = off.unwrap();
    assert_eq!(emission, Some(14));
    assert!(block.is_some());
}

#[test]
fn the_same_block_in_the_same_world_gives_an_equal_handle() {
    let fake = serving();
    fake.set_inventory(FakeInventory { held: Some(TORCH), selected_slot: Some(0), ..FakeInventory::default() });
    fake.set_off_hand(Some(TORCH));

    let [main, off] = hands_in_tick(&fake);

    assert_eq!(main.unwrap().1, off.unwrap().1);
}

#[test]
fn set_light_source_reaches_the_loader_and_none_clears() {
    let fake = serving();
    fake.set_inventory(FakeInventory { held: Some(TORCH), selected_slot: Some(0), ..FakeInventory::default() });
    let block = hands_in_tick(&fake)[0].unwrap().1;
    let lighting = fake.ctx().lighting();

    lighting.set_light_source(Some(light(block))).unwrap();
    assert_eq!(fake.light_source(), Some(light(block)));

    lighting.set_light_source(None).unwrap();
    assert_eq!(fake.light_source(), None);
    assert_eq!(fake.light_source_calls(), 2);
}

#[test]
fn a_block_of_a_left_world_is_stale() {
    let fake = serving();
    fake.set_inventory(FakeInventory { held: Some(TORCH), selected_slot: Some(0), ..FakeInventory::default() });
    let block = hands_in_tick(&fake)[0].unwrap().1;
    fake.set_epoch(2);

    assert_eq!(fake.ctx().lighting().set_light_source(Some(light(block))), Err(ApiError::Stale));
    assert_eq!(fake.light_source(), None);
}

#[test]
fn a_bad_level_or_a_refusal_sets_nothing() {
    let fake = serving();
    let lighting = fake.ctx().lighting();
    let bad = LightSource { level: 16, ..light(None) };
    assert!(matches!(lighting.set_light_source(Some(bad)), Err(ApiError::Loader(_))));
    fake.refuse_lighting(true);
    assert_eq!(lighting.set_light_source(Some(light(None))), Err(ApiError::Refused));
    assert_eq!(fake.light_source_calls(), 0);
}

#[test]
fn without_the_area_set_light_source_is_not_available() {
    let fake = FakeGame::new();
    fake.open_gate();
    assert!(!fake.ctx().supports(Area::LIGHTING_TERRAIN));
    assert_eq!(fake.ctx().lighting().set_light_source(None), Err(ApiError::NotAvailable));
}

#[test]
fn a_v5_loader_has_no_lighting_table() {
    let fake = FakeGame::with_v5_services();
    fake.offer_area(Area::LIGHTING_TERRAIN);
    assert!(!fake.ctx().supports(Area::LIGHTING_TERRAIN));
    assert_eq!(fake.ctx().lighting().set_light_source(None), Err(ApiError::NotAvailable));
}
