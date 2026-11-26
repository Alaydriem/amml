use std::sync::{Arc, Mutex};

use amml_abi::{ClientTick, ScreenRender};
use amml_api::testing::fake_screen::FakeScreen;
use amml_api::testing::{FakeClient, FakeEntry, FakeGame, FakeInventory, FakeItem};
use amml_api::{
    ApiError, Area, ContainerContents, Hand, HookOrder, ItemDrawFlags, Point, Screen, ScreenId, ScreenKind,
    ScreenRenderExt, ViewId,
};

const STONE: FakeItem = FakeItem { type_name: "minecraft:stone", count: 12, max_count: Some(64), ..FakeItem::EMPTY };
const PICKAXE: FakeItem =
    FakeItem { type_name: "minecraft:iron_pickaxe", count: 1, damage: 5, max_damage: 250, ..FakeItem::EMPTY };
const INSIDE: &[FakeEntry] = &[
    FakeEntry { slot: 1, item: STONE, bundle: None, bundle_weight: None },
    FakeEntry { slot: 0, item: PICKAXE, bundle: None, bundle_weight: None },
];
const BUNDLE: FakeItem = FakeItem { type_name: "minecraft:red_bundle", count: 1, ..FakeItem::EMPTY };
const SHULKER_ENTRIES: &[FakeEntry] = &[
    FakeEntry { slot: 0, item: STONE, bundle: None, bundle_weight: None },
    FakeEntry { slot: 3, item: BUNDLE, bundle: Some(INSIDE), bundle_weight: Some(40) },
];
const SHULKER: FakeItem =
    FakeItem { type_name: "minecraft:shulker_box", count: 1, contents: Some(SHULKER_ENTRIES), ..FakeItem::EMPTY };

fn in_world() -> FakeClient {
    FakeClient { local_player: true, world_active: true, top_screen: Screen::Other, ..FakeClient::default() }
}

fn serving(fake: &FakeGame, held: FakeItem) {
    fake.offer::<ClientTick>();
    fake.offer::<ScreenRender>();
    fake.offer_area(Area::CLIENT_READ);
    fake.offer_area(Area::INVENTORY_READ.at(4));
    fake.offer_area(Area::UI_SCREEN);
    fake.offer_area(Area::UI_DRAW.at(2));
    fake.open_gate();
    fake.set_client(in_world());
    fake.set_inventory(FakeInventory { held: Some(held), selected_slot: Some(0), ..FakeInventory::default() });
}

// The contents of the held item, read in a `ClientTick` listener.
fn read_contents(fake: &FakeGame) -> Option<ContainerContents> {
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let player = at.client().and_then(|client| client.local_player());
            let item = player.as_ref().and_then(|player| player.held_item(Hand::Main));
            *record.lock().unwrap() = Some(item.and_then(|item| item.container_contents()));
        })
        .unwrap();
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    seen.lock().unwrap().take().unwrap()
}

fn screen() -> FakeScreen {
    FakeScreen { kind: ScreenKind::SmallChest, gui_scale: Some(2.0), can_draw: true, ..FakeScreen::default() }
}

#[test]
fn contents_entries_match_the_fake() {
    let fake = FakeGame::new();
    serving(&fake, SHULKER);

    let contents = read_contents(&fake).unwrap();

    assert_eq!(contents.entries.len(), 2);
    let plain = &contents.entries[0];
    assert_eq!((plain.slot, plain.item.count, plain.item.max_count), (0, 12, Some(64)));
    assert_eq!(plain.item.type_name.as_deref(), Some("minecraft:stone"));
    assert!(plain.bundle.is_none() && plain.bundle_weight.is_none());
    let bundle = &contents.entries[1];
    assert_eq!((bundle.slot, bundle.bundle_weight), (3, Some(40)));
    assert_eq!(bundle.item.type_name.as_deref(), Some("minecraft:red_bundle"));
    let inner = bundle.bundle.as_ref().unwrap();
    let names: Vec<_> = inner.entries.iter().map(|entry| (entry.slot, entry.item.type_name.clone().unwrap())).collect();
    assert_eq!(names, vec![(1, "minecraft:stone".to_string()), (0, "minecraft:iron_pickaxe".to_string())]);
    assert_eq!(inner.entries[1].item.damage, Some(5));
    // Four handles: the two entries and the two inside the bundle.
    assert_eq!(fake.live_item_draws(), 4);
}

#[test]
fn an_item_without_contents_gives_none() {
    let fake = FakeGame::new();
    serving(&fake, STONE);

    assert!(read_contents(&fake).is_none());
    assert_eq!(fake.live_item_draws(), 0);
}

#[test]
fn a_loader_before_version_4_gives_none() {
    let fake = FakeGame::new();
    serving(&fake, SHULKER);
    fake.withdraw_area(Area::INVENTORY_READ.at(4));

    assert!(read_contents(&fake).is_none());
    assert_eq!(fake.live_item_draws(), 0);
}

#[test]
fn dropping_the_contents_frees_every_handle() {
    let fake = FakeGame::new();
    serving(&fake, SHULKER);

    let contents = read_contents(&fake).unwrap();
    assert_eq!(fake.live_item_draws(), 4);

    drop(contents);

    assert_eq!(fake.live_item_draws(), 0);
}

#[test]
fn dropping_one_entry_frees_only_its_handle() {
    let fake = FakeGame::new();
    serving(&fake, SHULKER);

    let mut contents = read_contents(&fake).unwrap();
    let removed = contents.entries.remove(0);
    drop(removed);

    assert_eq!(fake.live_item_draws(), 3);
}

#[test]
fn draw_item_records_the_item_and_passes_the_flags() {
    let fake = FakeGame::new();
    serving(&fake, SHULKER);
    let contents = Arc::new(Mutex::new(read_contents(&fake)));
    let held = contents.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let held = held.lock().unwrap();
            let entry = &held.as_ref().unwrap().entries[0];
            let draw = event.draw().unwrap();
            draw.item(&entry.draw, Point { x: 4.0, y: 5.0 }, ItemDrawFlags::FOIL_ONLY, 1.0, 2).unwrap();
            draw.item(&entry.draw, Point { x: 6.0, y: 7.0 }, ItemDrawFlags::NONE, 0.5, 2).unwrap();
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(fake.draws(), vec!["item minecraft:stone 4 5 1".to_string(), "item minecraft:stone 6 7 0".to_string()]);
}

#[test]
fn draw_item_of_a_stale_handle_is_stale() {
    let fake = FakeGame::new();
    serving(&fake, SHULKER);
    let contents = Arc::new(Mutex::new(read_contents(&fake)));
    let held = contents.clone();
    let results = Arc::new(Mutex::new(Vec::new()));
    let record = results.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let held = held.lock().unwrap();
            let entry = &held.as_ref().unwrap().entries[0];
            record.lock().unwrap().push(event.draw().unwrap().item(
                &entry.draw,
                Point::default(),
                ItemDrawFlags::NONE,
                1.0,
                0,
            ));
        })
        .unwrap();

    fake.leave_item_world();
    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(results.lock().unwrap().clone(), vec![Err(ApiError::Stale)]);
    assert!(fake.draws().is_empty());
    // Dropping a handle of the world that was left is allowed.
    contents.lock().unwrap().take();
    assert_eq!(fake.live_item_draws(), 0);
}

#[test]
fn use_after_world_leave_is_stale() {
    let fake = FakeGame::new();
    serving(&fake, SHULKER);
    let contents = read_contents(&fake).unwrap();

    fake.leave_item_world();

    assert_eq!(fake.live_item_draws(), 0);
    // A new read after the leave gives handles of the new generation, which draw.
    let again = read_contents(&fake).unwrap();
    let results = Arc::new(Mutex::new(Vec::new()));
    let record = results.clone();
    let entries = Arc::new(Mutex::new((contents, again)));
    let held = entries.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let held = held.lock().unwrap();
            let draw = event.draw().unwrap();
            for contents in [&held.0, &held.1] {
                record.lock().unwrap().push(draw.item(
                    &contents.entries[0].draw,
                    Point::default(),
                    ItemDrawFlags::NONE,
                    1.0,
                    0,
                ));
            }
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(results.lock().unwrap().clone(), vec![Err(ApiError::Stale), Ok(())]);
}

#[test]
fn drop_off_thread_queues_the_free() {
    let fake = FakeGame::new();
    serving(&fake, SHULKER);
    let contents = read_contents(&fake).unwrap();
    assert_eq!(fake.live_item_draws(), 4);

    // The handles are `Send`: they drop on another thread, where the loader only queues the free.
    std::thread::spawn(move || drop(contents)).join().unwrap();

    assert_eq!(fake.live_item_draws(), 4);
    assert_eq!(fake.pump_item_draws(), 4);
    assert_eq!(fake.live_item_draws(), 0);
}

#[test]
fn a_box_of_27_full_bundles_reads_in_full() {
    const SIXTY_FOUR: &[FakeEntry] = &[FakeEntry { slot: 0, item: STONE, bundle: None, bundle_weight: None }; 64];
    const FULL_BUNDLE: FakeEntry =
        FakeEntry { slot: 0, item: BUNDLE, bundle: Some(SIXTY_FOUR), bundle_weight: Some(64) };
    const BUNDLES: &[FakeEntry] = &[FULL_BUNDLE; 27];
    const BOX: FakeItem =
        FakeItem { type_name: "minecraft:shulker_box", count: 1, contents: Some(BUNDLES), ..FakeItem::EMPTY };
    let fake = FakeGame::new();
    serving(&fake, BOX);

    let contents = read_contents(&fake).unwrap();

    assert!(
        contents.entries.iter().all(|entry| entry.bundle.as_ref().is_some_and(|bundle| bundle.entries.len() == 64))
    );
    assert_eq!(fake.live_item_draws(), 27 + 27 * 64);
}

#[test]
fn a_mod_holds_at_most_4096_handles() {
    const FULL: &[FakeEntry] = &[FakeEntry { slot: 0, item: STONE, bundle: None, bundle_weight: None }; 27];
    const BIG: FakeItem =
        FakeItem { type_name: "minecraft:shulker_box", count: 1, contents: Some(FULL), ..FakeItem::EMPTY };
    let fake = FakeGame::new();
    serving(&fake, BIG);

    // 151 reads hold 4077 handles; the next would pass 4096.
    let held: Vec<_> = (0..151).map(|_| read_contents(&fake).unwrap()).collect();
    let refused = read_contents(&fake);

    assert_eq!(held.iter().map(|contents| contents.entries.len()).sum::<usize>(), 4077);
    assert!(refused.is_none());
    assert_eq!(fake.live_item_draws(), 4077);
}

#[test]
fn a_cap_hit_keeps_the_slots_and_leaves_out_the_contents_of_the_last_bundles() {
    const SIXTY_FOUR: &[FakeEntry] = &[FakeEntry { slot: 0, item: STONE, bundle: None, bundle_weight: None }; 64];
    const FULL_BUNDLE: FakeEntry =
        FakeEntry { slot: 0, item: BUNDLE, bundle: Some(SIXTY_FOUR), bundle_weight: Some(64) };
    const TEN: &[FakeEntry] = &[FakeEntry { slot: 0, item: STONE, bundle: None, bundle_weight: None }; 10];
    const SMALL_BUNDLE: FakeEntry = FakeEntry { slot: 0, item: BUNDLE, bundle: Some(TEN), bundle_weight: Some(10) };
    // The first bundle holds a bundle of 10 and 63 items: 10 more than 27 full bundles, which fill one read.
    const FIRST: &[FakeEntry] = &{
        let mut entries = [FakeEntry { slot: 0, item: STONE, bundle: None, bundle_weight: None }; 64];
        entries[0] = SMALL_BUNDLE;
        entries
    };
    const BUNDLES: &[FakeEntry] = &{
        let mut entries = [FULL_BUNDLE; 27];
        entries[0] = FakeEntry { slot: 0, item: BUNDLE, bundle: Some(FIRST), bundle_weight: Some(64) };
        entries
    };
    const BOX: FakeItem =
        FakeItem { type_name: "minecraft:shulker_box", count: 1, contents: Some(BUNDLES), ..FakeItem::EMPTY };
    let fake = FakeGame::new();
    serving(&fake, BOX);

    let contents = read_contents(&fake).unwrap();

    assert_eq!(contents.entries.len(), 27);
    let filled: Vec<bool> = contents.entries.iter().map(|entry| entry.bundle.is_some()).collect();
    assert!(filled[..26].iter().all(|filled| *filled));
    assert!(!filled[26]);
    // The 27 entries, the 64 + 10 of the first bundle and the 64 items of each of the 25 bundles after it.
    assert_eq!(fake.live_item_draws(), 27 + 74 + 25 * 64);
}
