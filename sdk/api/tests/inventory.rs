use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeClient, FakeGame, FakeInventory, FakeItem};
use amml_api::{Area, Hand, HookOrder, ItemSnapshot, Screen};

// What one `ClientTick` listener read: the selected slot, then the held item's name, count, damage and max damage,
// then the snapshot.
type Reading = (Option<u8>, Option<(Option<String>, u8, Option<i32>, Option<i32>)>, Option<ItemSnapshot>);

const PICKAXE: FakeItem =
    FakeItem { type_name: "minecraft:iron_pickaxe", count: 1, damage: 245, max_damage: 250, ..FakeItem::EMPTY };
const STONE: FakeItem = FakeItem {
    type_name: "minecraft:stone",
    count: 12,
    aux: Some(0),
    max_count: Some(64),
    is_glint: Some(false),
    identity: Some(0xbb81_b29a_8f70_34b9),
    ..FakeItem::EMPTY
};

fn in_world() -> FakeClient {
    FakeClient { local_player: true, world_active: true, top_screen: Screen::Other, ..FakeClient::default() }
}

fn serving(fake: &FakeGame, inventory: FakeInventory) {
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_READ);
    fake.offer_area(Area::INVENTORY_READ);
    fake.open_gate();
    fake.set_client(in_world());
    fake.set_inventory(inventory);
}

fn serving_v3(fake: &FakeGame, inventory: FakeInventory) {
    serving(fake, inventory);
    fake.offer_area(Area::INVENTORY_READ.at(3));
}

fn read_in_tick(fake: &FakeGame) -> Reading {
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let player = at.client().and_then(|client| client.local_player());
            let slot = player.as_ref().and_then(|player| player.inventory()).and_then(|inv| inv.selected_slot());
            let item = player.as_ref().and_then(|player| player.held_item(Hand::Main));
            let fields = item.as_ref().map(|item| (item.type_name(), item.count(), item.damage(), item.max_damage()));
            let snapshot = item.as_ref().map(|item| item.snapshot());
            *record.lock().unwrap() = Some((slot, fields, snapshot));
        })
        .unwrap();
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    seen.lock().unwrap().take().unwrap()
}

#[test]
fn the_held_item_and_the_selected_slot_are_read_in_a_listener() {
    let fake = FakeGame::new();
    serving(&fake, FakeInventory { held: Some(PICKAXE), selected_slot: Some(4), ..FakeInventory::default() });

    let (slot, fields, snapshot) = read_in_tick(&fake);

    assert_eq!(slot, Some(4));
    assert_eq!(fields, Some((Some("minecraft:iron_pickaxe".to_string()), 1, Some(245), Some(250))));
    let snapshot = snapshot.unwrap();
    assert_eq!(snapshot.type_name.as_deref(), Some("minecraft:iron_pickaxe"));
    assert_eq!((snapshot.count, snapshot.damage, snapshot.max_damage), (1, Some(245), Some(250)));
    assert_eq!((snapshot.aux, snapshot.max_count, snapshot.is_glint, snapshot.identity), (None, None, None, None));
}

#[test]
fn an_empty_hand_gives_no_item() {
    let fake = FakeGame::new();
    serving(&fake, FakeInventory { held: None, selected_slot: Some(0), ..FakeInventory::default() });

    let (slot, fields, _) = read_in_tick(&fake);

    assert_eq!(slot, Some(0));
    assert_eq!(fields, None);
}

#[test]
fn a_name_longer_than_the_first_buffer_is_read_whole() {
    let fake = FakeGame::new();
    let long: &'static str = Box::leak(format!("minecraft:{}", "x".repeat(300)).into_boxed_str());
    serving(
        &fake,
        FakeInventory {
            held: Some(FakeItem { type_name: long, ..PICKAXE }),
            selected_slot: Some(1),
            ..FakeInventory::default()
        },
    );

    let (_, fields, _) = read_in_tick(&fake);

    assert_eq!(fields.unwrap().0.as_deref(), Some(long));
}

#[test]
fn nothing_is_read_without_the_area() {
    let fake = FakeGame::new();
    serving(&fake, FakeInventory { held: Some(PICKAXE), selected_slot: Some(4), ..FakeInventory::default() });
    fake.withdraw_area(Area::INVENTORY_READ);

    let (slot, fields, _) = read_in_tick(&fake);

    assert_eq!((slot, fields), (None, None));
    assert!(!fake.ctx().supports(Area::INVENTORY_READ));
}

#[test]
fn a_loader_without_the_inventory_table_serves_nothing() {
    let fake = FakeGame::with_v4_services();
    serving(&fake, FakeInventory { held: Some(PICKAXE), selected_slot: Some(4), ..FakeInventory::default() });

    let (slot, fields, _) = read_in_tick(&fake);

    assert_eq!((slot, fields), (None, None));
    assert!(!fake.ctx().supports(Area::INVENTORY_READ));
}

#[test]
fn an_empty_off_hand_gives_no_item() {
    let fake = FakeGame::new();
    serving(&fake, FakeInventory { held: Some(PICKAXE), selected_slot: Some(4), ..FakeInventory::default() });
    let seen = Arc::new(Mutex::new(true));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let off = at.client().and_then(|client| client.local_player()).and_then(|p| p.held_item(Hand::Off));
            *record.lock().unwrap() = off.is_some();
        })
        .unwrap();

    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });

    assert!(!*seen.lock().unwrap());
}

// What a `ClientTick` listener read through the inventory: all 36 snapshots, then slot 3 as a view's name and count,
// then slot 40.
type SlotReading = (Option<[Option<ItemSnapshot>; 36]>, Option<(Option<String>, u8)>, bool);

fn read_slots_in_tick(fake: &FakeGame) -> SlotReading {
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let player = at.client().and_then(|client| client.local_player());
            let inventory = player.as_ref().and_then(|player| player.inventory()).unwrap();
            let view = inventory.slot(3).map(|item| (item.type_name(), item.count()));
            *record.lock().unwrap() = Some((inventory.slots(), view, inventory.slot(40).is_none()));
        })
        .unwrap();
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    seen.lock().unwrap().take().unwrap()
}

#[test]
fn slots_gives_36_snapshots_from_the_fake() {
    let fake = FakeGame::new();
    let mut slots = [None; 36];
    slots[0] = Some(PICKAXE);
    slots[3] = Some(STONE);
    slots[35] = Some(STONE);
    serving_v3(&fake, FakeInventory { selected_slot: Some(0), slots, ..FakeInventory::default() });

    let (snapshots, _, _) = read_slots_in_tick(&fake);

    let snapshots = snapshots.unwrap();
    assert_eq!(snapshots.iter().filter(|slot| slot.is_some()).count(), 3);
    let stone = snapshots[3].as_ref().unwrap();
    assert_eq!(stone.type_name.as_deref(), Some("minecraft:stone"));
    assert_eq!((stone.count, stone.aux, stone.max_count, stone.is_glint), (12, Some(0), Some(64), Some(false)));
    assert_eq!(stone.identity, Some(0xbb81_b29a_8f70_34b9));
    assert_eq!(snapshots[0].as_ref().unwrap().damage, Some(245));
    assert_eq!(snapshots[35], snapshots[3]);
    assert!(snapshots[1].is_none());
}

#[test]
fn slot_view_reads_type_name_and_count() {
    let fake = FakeGame::new();
    let mut slots = [None; 36];
    slots[3] = Some(STONE);
    serving_v3(&fake, FakeInventory { selected_slot: Some(0), slots, ..FakeInventory::default() });

    let (_, view, past_the_end_is_none) = read_slots_in_tick(&fake);

    assert_eq!(view, Some((Some("minecraft:stone".to_string()), 12)));
    assert!(past_the_end_is_none);
}

#[test]
fn identity_none_when_user_data_cannot_be_read() {
    let fake = FakeGame::new();
    let mut slots = [None; 36];
    slots[3] = Some(FakeItem { identity: None, ..STONE });
    serving_v3(&fake, FakeInventory { selected_slot: Some(0), slots, ..FakeInventory::default() });

    let (snapshots, _, _) = read_slots_in_tick(&fake);

    let snapshots = snapshots.unwrap();
    let stone = snapshots[3].as_ref().unwrap();
    assert_eq!(stone.identity, None);
    assert_eq!(stone.type_name.as_deref(), Some("minecraft:stone"));
}

#[test]
fn a_loader_that_offers_only_version_1_gives_no_slots() {
    let fake = FakeGame::new();
    let mut slots = [None; 36];
    slots[3] = Some(STONE);
    serving(&fake, FakeInventory { selected_slot: Some(0), slots, ..FakeInventory::default() });

    let (snapshots, view, _) = read_slots_in_tick(&fake);

    assert_eq!(snapshots, None);
    assert_eq!(view, None);
}

#[test]
fn the_change_stamp_reaches_the_view_and_the_snapshot_and_is_none_when_the_loader_cannot_read_it() {
    let fake = FakeGame::new();
    let boxed = FakeItem { type_name: "minecraft:shulker_box", count: 1, change_stamp: Some(0x51), ..FakeItem::EMPTY };
    serving(&fake, FakeInventory { held: Some(boxed), selected_slot: Some(0), ..FakeInventory::default() });
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let player = at.client().and_then(|client| client.local_player()).unwrap();
            let item = player.held_item(Hand::Main).unwrap();
            record.lock().unwrap().push((item.change_stamp(), item.snapshot().change_stamp));
        })
        .unwrap();

    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    fake.set_inventory(FakeInventory { held: Some(PICKAXE), selected_slot: Some(0), ..FakeInventory::default() });
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });

    assert_eq!(*seen.lock().unwrap(), [(Some(0x51), Some(0x51)), (None, None)]);
}
