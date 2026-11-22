use std::ffi::c_void;

use amml::{
    ActorAdapter, BlockReadAdapter, BlockStateAdapter, ClientAdapter, InventoryAdapter, ModLoader, SessionAdapter,
    WorldAdapter,
};
use amml_abi::{AmmlActorTable, AmmlApi, AmmlBlockInfo, AmmlBlockPos, AmmlServices, AmmlStatus, AmmlWorldTable};

use amml_abi::{AmmlBlockHandle, AmmlItemSnapshot, AmmlStr};
use amml_runtime::ModRegistry;

use crate::support::{
    ActorGame, BlockReadGame, BlockStateGame, ChunkGame, ChunkKind, ClientGame, EchoMod, EventsMod, GAME_CLIENT_A,
    Harness, SessionGame,
};

// The low 48 bits of a handle: the game address under the frame stamp.
const ADDRESS: usize = (1 << 48) - 1;

// Installs the adapters of the block reads over every stand-in entry but `without`.
fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&BlockReadGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    BlockReadAdapter::install(harness.state);
    harness
}

// A frame, then a joined world whose local player is the stand-in player.
fn join(harness: &Harness) {
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(ActorGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 1);
}

fn start_in_world() -> Harness {
    let harness = install(&[]);
    join(&harness);
    harness
}

fn table(harness: &Harness) -> AmmlWorldTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::world(services) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlWorldTable::read(table) }.unwrap()
}

// The block source handle the actor table gives for the stand-in player.
fn source_handle(harness: &Harness) -> usize {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::actor(services) }.unwrap();
    // SAFETY: as above.
    let table: AmmlActorTable = unsafe { AmmlActorTable::read(table) }.unwrap();
    let mut out: *const c_void = std::ptr::null();
    // SAFETY: the table is live and `out` is writable.
    let status =
        unsafe { (table.block_source.unwrap())(std::ptr::with_exposed_provenance(ActorGame::player()), &mut out) };
    assert_eq!(status, AmmlStatus::OK);
    out.addr()
}

fn pos(x: i32, y: i32, z: i32) -> AmmlBlockPos {
    AmmlBlockPos { x, y, z }
}

// `table.block_at`; the handle is `usize::MAX` while `out` was not written.
fn block_at(table: &AmmlWorldTable, source: usize, at: AmmlBlockPos) -> (AmmlStatus, usize) {
    let mut out = std::ptr::with_exposed_provenance::<c_void>(usize::MAX);
    // SAFETY: the table is live; the position and the output live for the call.
    let status =
        unsafe { (table.block_at.unwrap())(std::ptr::with_exposed_provenance(source), &raw const at, &mut out) };
    (status, out.addr())
}

fn block_info(table: &AmmlWorldTable, block: usize) -> (AmmlStatus, AmmlBlockInfo) {
    let mut out = AmmlBlockInfo { type_id: 0xDEAD, is_air: 9, is_interactive: 9, _reserved: [0; 2] };
    // SAFETY: the table is live and the output lives for the call.
    let status = unsafe { (table.block_info.unwrap())(std::ptr::with_exposed_provenance(block), &mut out) };
    (status, out)
}

#[test]
fn block_at_gives_the_game_block_of_the_position() {
    let harness = start_in_world();
    ChunkGame::set_block(3, 5, 10, ChunkKind::Stone);
    let table = table(&harness);
    let source = source_handle(&harness);

    let (status, stone) = block_at(&table, source, pos(3, 10, 5));
    let (_, air) = block_at(&table, source, pos(4, 10, 5));

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(stone & ADDRESS, ChunkGame::block(ChunkKind::Stone));
    assert_ne!(air & ADDRESS, stone & ADDRESS);
    assert_eq!(
        ChunkGame::source_asked(),
        [(ActorGame::block_source(), [3, 10, 5]), (ActorGame::block_source(), [4, 10, 5])]
    );
    let (_, info) = block_info(&table, stone);
    assert_eq!(WorldAdapter::get().unwrap().block_type_name(info.type_id).unwrap(), "minecraft:stone");
}

#[test]
fn block_of_an_older_frame_is_refused() {
    let harness = start_in_world();
    ChunkGame::set_block(1, 1, 1, ChunkKind::Stone);
    let table = table(&harness);
    let (_, block) = block_at(&table, source_handle(&harness), pos(1, 1, 1));
    assert_eq!(block_info(&table, block).0, AmmlStatus::OK);

    ClientGame::frame();

    assert_eq!(block_info(&table, block).0, AmmlStatus::REFUSED);
}

#[test]
fn air_needs_no_game_call() {
    let harness = start_in_world();
    ChunkGame::set_block(2, 2, 2, ChunkKind::Stone);
    let table = table(&harness);
    let source = source_handle(&harness);
    let (_, air) = block_at(&table, source, pos(0, 0, 0));
    let (_, stone) = block_at(&table, source, pos(2, 2, 2));

    let (air_status, air_info) = block_info(&table, air);
    assert_eq!((air_status, air_info.is_air, air_info.is_interactive), (AmmlStatus::OK, 1, 0));
    assert_eq!(BlockReadGame::interactive_calls(), 0);

    let (stone_status, stone_info) = block_info(&table, stone);
    assert_eq!((stone_status, stone_info.is_air, stone_info.is_interactive), (AmmlStatus::OK, 0, 1));
    assert_eq!(BlockReadGame::interactive_calls(), 1);
    assert_ne!(stone_info.type_id, air_info.type_id);
}

#[test]
fn built_over_passes_source_and_pos() {
    let harness = start_in_world();
    ChunkGame::set_block(6, 7, 8, ChunkKind::Stone);
    let table = table(&harness);
    let source = source_handle(&harness);
    let (_, block) = block_at(&table, source, pos(6, 8, 7));
    let at = pos(6, 8, 7);
    let mut out = 9u8;

    // SAFETY: the table is live; the position and the output live for the call.
    let status = unsafe {
        (table.block_built_over.unwrap())(
            std::ptr::with_exposed_provenance(source),
            std::ptr::with_exposed_provenance(block),
            &raw const at,
            &mut out,
        )
    };

    assert_eq!((status, out), (AmmlStatus::OK, 1));
    assert_eq!(
        BlockReadGame::built_over_calls(),
        [(
            ChunkGame::block_type(ChunkKind::Stone),
            ChunkGame::block(ChunkKind::Stone),
            ActorGame::block_source(),
            [6, 8, 7]
        )]
    );
}

#[test]
fn a_block_source_of_another_world_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let before = source_handle(&harness);

    ClientGame::frame();
    let stale = block_at(&table, before ^ (1 << 48), pos(0, 0, 0));

    assert_eq!(stale, (AmmlStatus::REFUSED, usize::MAX));
}

#[test]
fn block_reads_are_not_offered_without_get_block() {
    let harness = install(&["BlockSource.getBlock"]);
    join(&harness);

    assert!(!harness.state.areas().is_available("world.blocks", 1));
}

const FIRST_MOD: &str = "@events/events";
const SECOND_MOD: &str = "@echo/echo";

// A joined world over the block reads, the block states adapter and the inventory adapter, with the stand-in item
// path of `BlockStateGame` on the chunk stand-in block types, and two mods loaded.
fn start_full() -> Harness {
    let mut entries = BlockReadGame::entries(&["BlockType.asItemInstance", "ItemStackBase.dtor"]);
    let have: Vec<&str> = entries.iter().map(|(name, _)| *name).collect();
    let more: Vec<_> = BlockStateGame::entries()
        .into_iter()
        .chain(BlockStateGame::inventory_entries())
        .filter(|(name, _)| !have.contains(name))
        .collect();
    entries.extend(more);
    let harness = Harness::start_with_entries(&entries);
    ChunkGame::set_type_slot(2, BlockStateGame::as_item_address());
    BlockStateGame::register_type_name(ChunkGame::block_type(ChunkKind::Stone), "minecraft:stone");
    BlockStateGame::register_type_name(ChunkGame::block_type(ChunkKind::Dirt), "minecraft:dirt");
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    InventoryAdapter::install(harness.state);
    BlockStateAdapter::install(harness.state);
    BlockReadAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    harness.install_mod("echo", "echo", &EchoMod::build("plain", &[]), "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    join(&harness);
    harness
}

fn block_item(table: &AmmlWorldTable, block: usize) -> (AmmlStatus, AmmlItemSnapshot, String) {
    let mut snapshot = AmmlItemSnapshot::empty();
    let (mut buffer, mut len) = ([0u8; 64], 0usize);
    // SAFETY: the table is live; the snapshot, the buffer and the length are live locals.
    let status = unsafe {
        (table.block_item.unwrap())(
            std::ptr::with_exposed_provenance(block),
            &mut snapshot,
            buffer.as_mut_ptr(),
            buffer.len(),
            &mut len,
        )
    };
    (status, snapshot, String::from_utf8_lossy(&buffer[..len.min(64)]).into_owned())
}

fn handle_of(table: &AmmlWorldTable, module: &str, block: usize) -> (AmmlStatus, AmmlBlockHandle) {
    let mut out = AmmlBlockHandle::NONE;
    // SAFETY: the table is live; the module text and the output live for the call.
    let status = unsafe {
        (table.block_handle_of.unwrap())(AmmlStr::from_str(module), std::ptr::with_exposed_provenance(block), &mut out)
    };
    (status, out)
}

fn handle_name(table: &AmmlWorldTable, block: AmmlBlockHandle) -> (AmmlStatus, String) {
    let (mut buffer, mut len) = ([0u8; 64], 0usize);
    // SAFETY: the table is live; the buffer and the length are live locals.
    let status = unsafe { (table.block_handle_name.unwrap())(block, buffer.as_mut_ptr(), buffer.len(), &mut len) };
    (status, String::from_utf8_lossy(&buffer[..len.min(64)]).into_owned())
}

#[test]
fn block_item_builds_the_stack_reads_it_and_destroys_it_once() {
    let harness = start_full();
    ChunkGame::set_block(3, 5, 10, ChunkKind::Stone);
    let table = table(&harness);
    let (_, block) = block_at(&table, source_handle(&harness), pos(3, 10, 5));

    let (status, snapshot, name) = block_item(&table, block);

    assert_eq!((status, snapshot.count, name.as_str()), (AmmlStatus::OK, 1, "minecraft:stone"));
    let calls = BlockStateGame::as_item_calls();
    assert_eq!(calls.len(), 1);
    let (block_type, buffer, game_block, actor) = calls[0];
    assert_eq!(
        (block_type, game_block, actor),
        (ChunkGame::block_type(ChunkKind::Stone), ChunkGame::block(ChunkKind::Stone), 0)
    );
    assert_eq!(BlockStateGame::item_dtors(), vec![buffer]);
}

#[test]
fn block_item_of_an_older_frame_is_refused_and_builds_nothing() {
    let harness = start_full();
    ChunkGame::set_block(3, 5, 10, ChunkKind::Stone);
    let table = table(&harness);
    let (_, block) = block_at(&table, source_handle(&harness), pos(3, 10, 5));

    ClientGame::frame();

    assert_eq!(block_item(&table, block).0, AmmlStatus::REFUSED);
    assert!(BlockStateGame::as_item_calls().is_empty());
}

#[test]
fn block_item_without_the_block_states_adapter_is_not_available() {
    let harness = start_in_world();
    ChunkGame::set_block(3, 5, 10, ChunkKind::Stone);
    let table = table(&harness);
    let (_, block) = block_at(&table, source_handle(&harness), pos(3, 10, 5));

    assert_eq!(block_item(&table, block).0, AmmlStatus::NOT_AVAILABLE);
    assert!(BlockStateGame::as_item_calls().is_empty());
}

#[test]
fn handle_of_interns_per_mod_and_per_game_block() {
    let harness = start_full();
    ChunkGame::set_block(1, 1, 1, ChunkKind::Stone);
    ChunkGame::set_block(2, 2, 2, ChunkKind::Stone);
    ChunkGame::set_block(3, 3, 3, ChunkKind::Dirt);
    let table = table(&harness);
    let source = source_handle(&harness);
    let (_, first) = block_at(&table, source, pos(1, 1, 1));
    let (_, same) = block_at(&table, source, pos(2, 2, 2));
    let (_, other) = block_at(&table, source, pos(3, 3, 3));

    let (status, mine) = handle_of(&table, FIRST_MOD, first);
    let (_, mine_again) = handle_of(&table, FIRST_MOD, same);
    let (_, mine_other) = handle_of(&table, FIRST_MOD, other);
    let (_, theirs) = handle_of(&table, SECOND_MOD, first);

    assert_eq!(status, AmmlStatus::OK);
    assert_ne!(mine, AmmlBlockHandle::NONE);
    assert_eq!(mine, mine_again);
    assert_ne!(mine, mine_other);
    assert_ne!(mine, theirs);
    assert_eq!(handle_name(&table, mine), (AmmlStatus::OK, "minecraft:stone".to_string()));
    assert_eq!(handle_of(&table, "@no/such", first).0, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn handle_of_is_stale_after_the_world_is_left() {
    let harness = start_full();
    ChunkGame::set_block(1, 1, 1, ChunkKind::Stone);
    let table = table(&harness);
    let (_, block) = block_at(&table, source_handle(&harness), pos(1, 1, 1));
    let (_, mine) = handle_of(&table, FIRST_MOD, block);
    assert_eq!(handle_name(&table, mine).0, AmmlStatus::OK);

    SessionGame::exit();
    SessionGame::frame();
    SessionGame::frame();

    assert_eq!(handle_name(&table, mine).0, AmmlStatus::STALE);
}
