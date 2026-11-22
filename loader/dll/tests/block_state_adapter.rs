use amml::{BlockStateAdapter, InventoryAdapter, ModLoader, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlBlockHandle, AmmlBlockMatch, AmmlItemSnapshot, AmmlServices, AmmlStateEntry, AmmlStateValue,
    AmmlStatus, AmmlStr, AmmlWorldTable,
};
use amml_runtime::ModRegistry;

use crate::support::{BlockStateGame, CLIENT_A, EventsMod, FakeValue, Harness, HeapProbe, SessionGame};

const MODULE: &str = "@events/events";
const STAIRS: &str = "minecraft:oak_stairs";

// A harness with the session adapter, the block states adapter over the stand-in registry and one loaded mod.
fn start() -> Harness {
    start_with(false)
}

// As `start`, with the inventory adapter over the stand-in stack mirror when `inventory` is true.
fn start_with(inventory: bool) -> Harness {
    let mut entries = SessionGame::entries(&[]);
    entries.extend(BlockStateGame::entries());
    if inventory {
        entries.extend(BlockStateGame::inventory_entries());
    }
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    BlockStateAdapter::install(harness.state);
    if inventory {
        InventoryAdapter::install(harness.state);
    }
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    BlockStateGame::add_type(
        STAIRS,
        &[
            &[("upside_down_bit", FakeValue::Byte(0)), ("weirdo_direction", FakeValue::Int(0))],
            &[("upside_down_bit", FakeValue::Byte(0)), ("weirdo_direction", FakeValue::Int(2))],
            &[("upside_down_bit", FakeValue::Byte(1)), ("weirdo_direction", FakeValue::Int(2))],
        ],
    );
    harness
}

// A harness in a joined world, on the game thread.
fn start_in_world() -> Harness {
    join(start())
}

fn join(harness: Harness) -> Harness {
    SessionGame::frame();
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    assert_ne!(harness.state.events().gate().epoch().0, 0);
    harness
}

fn table(harness: &Harness) -> &'static AmmlWorldTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    unsafe { &*AmmlServices::world(services).unwrap() }
}

fn int(name: &'static str, value: i32) -> AmmlStateEntry {
    AmmlStateEntry {
        name: AmmlStr::from_str(name),
        value: AmmlStateValue { kind: AmmlStateValue::INT, _reserved: [0; 3], int: value, text: AmmlStr::EMPTY },
    }
}

fn flag(name: &'static str, value: bool) -> AmmlStateEntry {
    AmmlStateEntry {
        name: AmmlStr::from_str(name),
        value: AmmlStateValue {
            kind: AmmlStateValue::BOOL,
            _reserved: [0; 3],
            int: i32::from(value),
            text: AmmlStr::EMPTY,
        },
    }
}

fn from_state(table: &AmmlWorldTable, name: &str, states: &[AmmlStateEntry]) -> (AmmlStatus, AmmlBlockMatch) {
    let mut found = AmmlBlockMatch { block: AmmlBlockHandle::NONE, exact: 0, _reserved: [0; 7] };
    // SAFETY: the table is live; the texts, the entries and the output live for the call.
    let status = unsafe {
        (table.block_from_state.unwrap())(
            AmmlStr::from_str(MODULE),
            AmmlStr::from_str(name),
            states.as_ptr(),
            states.len() as u32,
            &mut found,
        )
    };
    (status, found)
}

fn rotated(table: &AmmlWorldTable, block: AmmlBlockHandle, quarters: u8) -> (AmmlStatus, AmmlBlockHandle) {
    let mut out = AmmlBlockHandle::NONE;
    // SAFETY: the table is live; the output lives for the call.
    let status = unsafe { (table.block_rotated.unwrap())(AmmlStr::from_str(MODULE), block, quarters, &mut out) };
    (status, out)
}

fn name(table: &AmmlWorldTable, block: AmmlBlockHandle) -> (AmmlStatus, String) {
    let (mut buffer, mut len) = ([0u8; 64], 0usize);
    // SAFETY: the table is live; the buffer and the length are live locals.
    let status = unsafe { (table.block_handle_name.unwrap())(block, buffer.as_mut_ptr(), buffer.len(), &mut len) };
    (status, String::from_utf8_lossy(&buffer[..len.min(64)]).into_owned())
}

#[test]
fn exact_states_give_that_block_with_exact_true() {
    let harness = start_in_world();
    let table = table(&harness);

    let (status, found) = from_state(table, STAIRS, &[flag("upside_down_bit", true), int("weirdo_direction", 2)]);
    let (_, partial) = from_state(table, STAIRS, &[int("weirdo_direction", 2)]);

    assert_eq!((status, found.exact), (AmmlStatus::OK, 1));
    assert_ne!(found.block, AmmlBlockHandle::NONE);
    assert_eq!(partial.exact, 1);
    assert_ne!(partial.block, found.block);
    assert_eq!(name(table, found.block), (AmmlStatus::OK, STAIRS.to_string()));
}

#[test]
fn unknown_states_fall_back_to_default_with_exact_false() {
    let harness = start_in_world();
    let table = table(&harness);

    let (status, found) = from_state(table, STAIRS, &[int("no_such_state", 3)]);
    let (_, wrong_type) = from_state(table, STAIRS, &[flag("weirdo_direction", true)]);
    let (_, default) = from_state(table, STAIRS, &[]);

    assert_eq!((status, found.exact), (AmmlStatus::OK, 0));
    assert_eq!(found.block, default.block);
    assert_eq!(default.exact, 1);
    assert_eq!(wrong_type.exact, 0);
}

#[test]
fn the_version_comes_from_the_game_block_tag() {
    let harness = start_in_world();

    from_state(table(&harness), STAIRS, &[int("weirdo_direction", 2)]);

    assert_eq!(BlockStateGame::versions(), vec![BlockStateGame::VERSION]);
    let lookups = BlockStateGame::lookups();
    assert_eq!(lookups.len(), 1);
    assert_eq!(lookups[0].0, STAIRS);
    assert_ne!(lookups[0].1, 0);
}

#[test]
fn unknown_name_gives_none() {
    let harness = start_in_world();
    let table = table(&harness);

    let (status, found) = from_state(table, "minecraft:no_such_block", &[int("weirdo_direction", 2)]);
    let (empty_status, empty) = from_state(table, "", &[]);

    assert_eq!((status, found.block), (AmmlStatus::OK, AmmlBlockHandle::NONE));
    assert_eq!((empty_status, empty.block), (AmmlStatus::OK, AmmlBlockHandle::NONE));
}

#[test]
fn tag_is_freed_on_every_path() {
    let harness = start_in_world();
    let table = table(&harness);

    from_state(table, STAIRS, &[flag("upside_down_bit", true), int("weirdo_direction", 2)]);
    from_state(table, STAIRS, &[int("no_such_state", 3)]);
    from_state(table, "minecraft:no_such_block", &[]);
    BlockStateGame::find_nothing();
    let (status, nothing) = from_state(table, STAIRS, &[]);

    assert_eq!((status, nothing.block), (AmmlStatus::OK, AmmlBlockHandle::NONE));
    let (made, destroyed, live) = BlockStateGame::tags();
    assert_eq!((made, destroyed, live), (3, 3, 0));
    // The allocator can give a freed tag's address to the next tag, so each address is freed at least as often as it was made.
    let (freed, made_tags) = (HeapProbe::freed(), BlockStateGame::made_tags());
    for tag in &made_tags {
        let made = made_tags.iter().filter(|other| *other == tag).count();
        assert!(freed.iter().filter(|block| **block == (*tag, 0x18)).count() >= made, "tag {tag:#x}");
    }
}

#[test]
fn same_block_gives_an_equal_handle() {
    let harness = start_in_world();
    let table = table(&harness);

    let (_, first) = from_state(table, STAIRS, &[flag("upside_down_bit", true), int("weirdo_direction", 2)]);
    let (_, again) = from_state(table, STAIRS, &[int("weirdo_direction", 2), flag("upside_down_bit", true)]);
    let (_, other) = from_state(table, STAIRS, &[]);

    assert_eq!(first.block, again.block);
    assert_ne!(first.block, other.block);
}

#[test]
fn rotated_takes_quarters_mod_4() {
    let harness = start_in_world();
    let table = table(&harness);
    BlockStateGame::set_rotation(BlockStateGame::block(STAIRS, 1), 1, BlockStateGame::block(STAIRS, 0));
    let (_, east) = from_state(table, STAIRS, &[int("weirdo_direction", 2)]);
    let (_, default) = from_state(table, STAIRS, &[]);

    let (status, turned) = rotated(table, east.block, 5);
    let (same_status, same) = rotated(table, east.block, 4);
    let (_, unknown) = rotated(table, default.block, 2);

    assert_eq!((status, turned), (AmmlStatus::OK, default.block));
    assert_eq!((same_status, same), (AmmlStatus::OK, east.block));
    assert_eq!(unknown, default.block);
    let east_address = BlockStateGame::block(STAIRS, 1);
    let default_address = BlockStateGame::block(STAIRS, 0);
    assert_eq!(BlockStateGame::turns(), vec![(east_address, 1, 0), (default_address, 2, 0)]);
}

#[test]
fn off_the_game_thread_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let (_, found) = from_state(table, STAIRS, &[]);

    let (status, rotation) = std::thread::spawn(move || {
        let (status, _) = from_state(table, STAIRS, &[]);
        (status, rotated(table, found.block, 1).0)
    })
    .join()
    .unwrap();

    assert_eq!((status, rotation), (AmmlStatus::REFUSED, AmmlStatus::REFUSED));
    assert_eq!(BlockStateGame::tags().0, 1);
}

#[test]
fn before_a_world_is_joined_the_call_is_not_ready() {
    let harness = start();
    SessionGame::frame();

    assert_eq!(from_state(table(&harness), STAIRS, &[]).0, AmmlStatus::NOT_READY);
    assert_eq!(BlockStateGame::tags().0, 0);
}

#[test]
fn a_handle_from_a_left_world_is_stale() {
    let harness = start_in_world();
    let table = table(&harness);
    let (_, found) = from_state(table, STAIRS, &[]);

    SessionGame::exit();
    SessionGame::frame();
    SessionGame::frame();

    assert_eq!(name(table, found.block).0, AmmlStatus::STALE);
}

fn item(table: &AmmlWorldTable, block: AmmlBlockHandle) -> (AmmlStatus, AmmlItemSnapshot, String) {
    let (mut snapshot, mut buffer, mut len) = (AmmlItemSnapshot::empty(), [0u8; 64], 0usize);
    // SAFETY: the table is live; the snapshot, the buffer and the length are live locals.
    let status = unsafe {
        (table.block_handle_item.unwrap())(block, &mut snapshot, buffer.as_mut_ptr(), buffer.len(), &mut len)
    };
    (status, snapshot, String::from_utf8_lossy(&buffer[..len.min(64)]).into_owned())
}

#[test]
fn item_builds_the_stack_reads_it_and_destroys_it_once() {
    let harness = join(start_with(true));
    let table = table(&harness);
    let (_, found) = from_state(table, STAIRS, &[int("weirdo_direction", 2)]);

    let (status, snapshot, name) = item(table, found.block);

    assert_eq!((status, snapshot.count, name.as_str()), (AmmlStatus::OK, 1, STAIRS));
    let calls = BlockStateGame::as_item_calls();
    assert_eq!(calls.len(), 1);
    let (block_type, buffer, block, actor) = calls[0];
    assert_eq!((block_type, block, actor), (BlockStateGame::block_type(STAIRS), BlockStateGame::block(STAIRS, 1), 0));
    assert_eq!(BlockStateGame::item_dtors(), vec![buffer]);
}

#[test]
fn item_without_the_inventory_adapter_is_not_available() {
    let harness = start_in_world();
    let table = table(&harness);
    let (_, found) = from_state(table, STAIRS, &[]);

    assert_eq!(item(table, found.block).0, AmmlStatus::NOT_AVAILABLE);
    assert!(BlockStateGame::as_item_calls().is_empty());
    assert!(BlockStateGame::item_dtors().is_empty());
}
