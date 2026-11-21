use std::sync::{Arc, Mutex};

use amml_abi::{ContainerSlotHover, ContainerSlotUnhover, ScreenRender};
use amml_api::testing::fake_screen::FakeScreen;
use amml_api::testing::{FakeClient, FakeGame, FakeInventory, FakeItem, FakeSlots};
use amml_api::{
    Amount, ApiError, Area, Collection, ContainerScreen, ContainerSlotHoverExt, HookOrder, ScreenId, ScreenKind,
    ScreenRenderExt, SlotAt, ViewId,
};

const STONE: FakeItem = FakeItem {
    type_name: "minecraft:stone",
    count: 12,
    aux: Some(0),
    max_count: Some(64),
    is_glint: Some(false),
    identity: Some(0xbb81_b29a_8f70_34b9),
    ..FakeItem::EMPTY
};
const DIAMOND: FakeItem = FakeItem { type_name: "minecraft:diamond", count: 3, max_count: Some(64), ..FakeItem::EMPTY };

fn serving() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<ScreenRender>();
    fake.offer::<ContainerSlotHover>();
    fake.offer::<ContainerSlotUnhover>();
    fake.offer_area(Area::UI_SCREEN.at(2));
    fake.offer_area(Area::INVENTORY_READ.at(3));
    fake.open_gate();
    fake
}

fn slots(items: &[(usize, FakeItem)], len: usize) -> Vec<Option<FakeItem>> {
    let mut all = vec![None; len];
    for (index, item) in items {
        all[*index] = Some(*item);
    }
    all
}

// A small chest with a stone in container slot 0 and a diamond in hotbar slot 3.
fn chest() -> FakeScreen {
    FakeScreen {
        kind: ScreenKind::SmallChest,
        slots: FakeSlots {
            container: slots(&[(0, STONE)], 27),
            hotbar: slots(&[(3, DIAMOND)], 9),
            inventory: slots(&[], 27),
        },
        ..FakeScreen::default()
    }
}

// Runs `body` in a `ScreenRender` listener for `screen` and gives what it returned.
fn on_container<T: Send + 'static>(
    fake: &FakeGame,
    screen: &FakeScreen,
    body: impl Fn(&ContainerScreen<'_>) -> T + Send + 'static,
) -> Option<T> {
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            *record.lock().unwrap() = event.container_screen().map(|container| body(&container));
        })
        .unwrap();
    let dispatch = fake.raise_screen_render(ScreenId(7), ViewId(9), screen);
    assert!(!dispatch.panicked);
    seen.lock().unwrap().take()
}

fn names(items: Vec<Option<amml_api::ItemSnapshot>>) -> Vec<Option<String>> {
    items.into_iter().map(|item| item.and_then(|item| item.type_name)).collect()
}

#[test]
fn slots_of_a_small_chest_are_27() {
    let fake = serving();

    let read = on_container(&fake, &chest(), |container| {
        (
            container.id(),
            container.kind(),
            container.slots(Collection::Container),
            container.slots(Collection::Hotbar),
            container.slots(Collection::Inventory),
            container.slots(Collection::Other),
        )
    })
    .unwrap();

    assert_eq!(read.0, ScreenId(7));
    assert_eq!(read.1, ScreenKind::SmallChest);
    let container = read.2.unwrap();
    assert_eq!(container.len(), 27);
    let first = container[0].as_ref().unwrap();
    assert_eq!((first.type_name.as_deref(), first.count), (Some("minecraft:stone"), 12));
    assert!(container[1..].iter().all(Option::is_none));
    assert_eq!(read.3.unwrap().len(), 9);
    assert_eq!(read.4.unwrap().len(), 27);
    assert_eq!(read.5, None);
}

#[test]
fn a_screen_that_is_not_a_container_gives_none() {
    let fake = serving();

    let read = on_container(&fake, &FakeScreen { kind: ScreenKind::Other, ..chest() }, |_| ());

    assert_eq!(read, None);
}

#[test]
fn auto_place_is_recorded_and_applied() {
    let fake = serving();

    let read = on_container(&fake, &chest(), |container| {
        let moved = container.auto_place(SlotAt { collection: Collection::Hotbar, index: 3 });
        (
            moved,
            names(container.slots(Collection::Container).unwrap()),
            names(container.slots(Collection::Hotbar).unwrap()),
        )
    })
    .unwrap();

    assert_eq!(read.0, Ok(()));
    // The diamond goes to the first free container slot (slot 1: the stone holds slot 0).
    assert_eq!(read.1[1].as_deref(), Some("minecraft:diamond"));
    assert!(read.2.iter().all(Option::is_none));
    assert_eq!(fake.screen_actions(), ["auto_place hotbar 3"]);
}

#[test]
fn take_all_and_place_all_move_through_the_cursor() {
    let fake = serving();

    let read = on_container(&fake, &chest(), |container| {
        let take = container.take_all(SlotAt { collection: Collection::Container, index: 0 });
        let cursor = container.cursor().unwrap().and_then(|item| item.type_name);
        let place = container.place_all(SlotAt { collection: Collection::Inventory, index: 5 });
        let after = container.cursor().unwrap();
        (take, cursor, place, after.is_none(), names(container.slots(Collection::Inventory).unwrap())[5].clone())
    })
    .unwrap();

    assert_eq!(read.0, Ok(()));
    assert_eq!(read.1.as_deref(), Some("minecraft:stone"));
    assert_eq!(read.2, Ok(()));
    assert!(read.3);
    assert_eq!(read.4.as_deref(), Some("minecraft:stone"));
    assert_eq!(fake.screen_actions(), ["take_all container 0", "place_all inventory 5"]);
}

#[test]
fn drops_remove_the_amount_and_are_recorded() {
    let fake = serving();

    let read = on_container(&fake, &chest(), |container| {
        let one = container.drop_slot(SlotAt { collection: Collection::Container, index: 0 }, Amount::One);
        let left = container.slots(Collection::Container).unwrap()[0].as_ref().map(|item| item.count);
        let all = container.drop_slot(SlotAt { collection: Collection::Hotbar, index: 3 }, Amount::Whole);
        let taken = container.take_all(SlotAt { collection: Collection::Container, index: 0 });
        let cursor = container.drop_cursor(Amount::Count(5));
        let cursor_left = container.cursor().unwrap().map(|item| item.count);
        (one, left, all, taken, cursor, cursor_left, container.slots(Collection::Hotbar).unwrap()[3].is_none())
    })
    .unwrap();

    assert_eq!(read, (Ok(()), Some(11), Ok(()), Ok(()), Ok(()), Some(6), true));
    assert_eq!(
        fake.screen_actions(),
        ["drop_slot container 0 one", "drop_slot hotbar 3 whole", "take_all container 0", "drop_cursor 5"]
    );
}

#[test]
fn swap_gives_true_when_both_slots_hold_items() {
    let fake = serving();
    let screen = FakeScreen { slots: FakeSlots { inventory: slots(&[(4, DIAMOND)], 27), ..chest().slots }, ..chest() };

    let read = on_container(&fake, &screen, |container| {
        let container_slot = SlotAt { collection: Collection::Container, index: 0 };
        let inventory_slot = SlotAt { collection: Collection::Inventory, index: 4 };
        let both = container.swap(container_slot, inventory_slot);
        let after = (
            container.slots(Collection::Container).unwrap()[0].clone().and_then(|item| item.type_name),
            container.slots(Collection::Inventory).unwrap()[4].clone().and_then(|item| item.type_name),
        );
        let with_empty = container.swap(container_slot, SlotAt { collection: Collection::Inventory, index: 9 });
        (both, after, with_empty)
    })
    .unwrap();

    assert_eq!(read.0, Ok(true));
    assert_eq!(read.1, (Some("minecraft:diamond".to_string()), Some("minecraft:stone".to_string())));
    assert_eq!(read.2, Ok(false));
    assert_eq!(fake.screen_actions()[0], "swap container 0 inventory 4");
}

#[test]
fn a_slot_outside_the_collection_is_refused_by_the_loader() {
    let fake = serving();

    let read = on_container(&fake, &chest(), |container| {
        (
            container.auto_place(SlotAt { collection: Collection::Container, index: 27 }),
            container.auto_place(SlotAt { collection: Collection::Other, index: 0 }),
            container.visual_item(SlotAt { collection: Collection::Hotbar, index: -1 }).is_none(),
        )
    })
    .unwrap();

    let invalid = ApiError::Loader(amml_abi::AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(read, (Err(invalid), Err(invalid), true));
    assert!(fake.screen_actions().is_empty());
}

#[test]
fn visual_item_reads_like_an_inventory_item() {
    let fake = serving();

    let read = on_container(&fake, &chest(), |container| {
        let stone = container.visual_item(SlotAt { collection: Collection::Container, index: 0 }).unwrap();
        let empty = container.visual_item(SlotAt { collection: Collection::Container, index: 1 });
        (
            stone.type_name(),
            stone.count(),
            stone.max_count(),
            stone.aux(),
            stone.identity(),
            stone.snapshot().type_name,
            empty.is_none(),
        )
    })
    .unwrap();

    assert_eq!(read.0.as_deref(), Some("minecraft:stone"));
    assert_eq!((read.1, read.2, read.3, read.4), (12, Some(64), Some(0), Some(0xbb81_b29a_8f70_34b9)));
    assert_eq!(read.5.as_deref(), Some("minecraft:stone"));
    assert!(read.6);
}

#[test]
fn a_loader_without_the_container_area_gives_no_container_screen() {
    let fake = FakeGame::new();
    fake.offer::<ScreenRender>();
    fake.offer_area(Area::UI_SCREEN);
    fake.open_gate();

    let read = on_container(&fake, &chest(), |container| container.slots(Collection::Container));

    assert_eq!(read, Some(None));
}

#[test]
fn hover_ext_reads_screen_and_slot() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _hover = fake
        .ctx()
        .events()
        .listen::<ContainerSlotHover>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(("hover", event.screen(), event.slot()));
        })
        .unwrap();
    let record = seen.clone();
    let _unhover = fake
        .ctx()
        .events()
        .listen::<ContainerSlotUnhover>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(("unhover", event.screen(), event.slot()));
        })
        .unwrap();
    let slot = SlotAt { collection: Collection::Hotbar, index: 2 };

    fake.raise_hover(ScreenId(5), slot);
    fake.raise_unhover(ScreenId(5), SlotAt { collection: Collection::Other, index: 0 });

    assert_eq!(
        *seen.lock().unwrap(),
        [("hover", ScreenId(5), slot), ("unhover", ScreenId(5), SlotAt { collection: Collection::Other, index: 0 }),]
    );
}

#[test]
fn a_stack_read_before_a_move_is_refused_after_it() {
    let fake = serving();

    let read = on_container(&fake, &chest(), |container| {
        let slot = SlotAt { collection: Collection::Container, index: 0 };
        let before = container.visual_item(slot).unwrap();
        let count_before = before.count();
        container.take_all(slot).unwrap();
        // The old handle reads nothing now; a new read of the emptied slot gives no stack.
        let stale = (before.count(), before.type_name());
        let after = container.visual_item(slot).is_none();
        let cursor = container.cursor().unwrap().map(|item| item.count);
        (count_before, stale, after, cursor)
    })
    .unwrap();

    assert_eq!(read, (12, (0, None), true, Some(12)));
}

// Runs `body` in a `ScreenRender` listener of `screen` with the inventory of the local player and its container screen.
fn on_container_with_inventory<T: Send + 'static>(
    fake: &FakeGame,
    screen: &FakeScreen,
    body: impl Fn(&amml_api::Inventory<'_>, &ContainerScreen<'_>) -> T + Send + 'static,
) -> Option<T> {
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, at| {
            let inventory = at.client().and_then(|client| client.local_player()).and_then(|player| player.inventory());
            let (Some(inventory), Some(container)) = (inventory, event.container_screen()) else { return };
            *record.lock().unwrap() = Some(body(&inventory, &container));
        })
        .unwrap();
    let dispatch = fake.raise_screen_render(ScreenId(7), ViewId(9), screen);
    assert!(!dispatch.panicked);
    seen.lock().unwrap().take()
}

#[test]
fn an_inventory_stack_read_before_a_container_move_is_refused_after_it() {
    let fake = serving();
    fake.offer_area(Area::CLIENT_READ);
    fake.offer_area(Area::INVENTORY_READ);
    fake.set_client(FakeClient { local_player: true, world_active: true, ..FakeClient::default() });
    let mut inventory = FakeInventory::default();
    inventory.slots[0] = Some(STONE);
    fake.set_inventory(inventory);

    let read = on_container_with_inventory(&fake, &chest(), |inventory, container| {
        let before = inventory.slot(0).unwrap();
        let count_before = before.count();
        container.take_all(SlotAt { collection: Collection::Container, index: 0 }).unwrap();
        // The old handle reads nothing now; a read of the slot again gives a new handle.
        let stale = (before.count(), before.type_name());
        let again = inventory.slot(0).map(|view| view.count());
        (count_before, stale, again)
    })
    .unwrap();

    assert_eq!(read, (12, (0, None), Some(12)));
}

#[test]
fn the_moves_of_a_render_apply_to_the_screen_the_test_holds() {
    let fake = serving();
    let mut screen = chest();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            if let Some(container) = event.container_screen() {
                let _ = container.auto_place(SlotAt { collection: Collection::Hotbar, index: 3 });
            }
        })
        .unwrap();

    let dispatch = fake.raise_screen_render_mut(ScreenId(7), ViewId(9), &mut screen);

    assert!(!dispatch.panicked);
    assert_eq!(screen.slots.container[1].map(|item| item.type_name), Some("minecraft:diamond"));
    assert!(screen.slots.hotbar[3].is_none());
    // The next render starts from the moved screen: the hotbar slot is empty, so the same move places nothing more.
    fake.raise_screen_render_mut(ScreenId(7), ViewId(9), &mut screen);
    assert_eq!(screen.slots.container[1].map(|item| item.type_name), Some("minecraft:diamond"));
    assert!(screen.slots.container[2].is_none());
    // `raise_screen_render` keeps working on a copy.
    let copy = chest();
    fake.raise_screen_render(ScreenId(7), ViewId(9), &copy);
    assert_eq!(copy.slots.hotbar[3].map(|item| item.type_name), Some("minecraft:diamond"));
}
