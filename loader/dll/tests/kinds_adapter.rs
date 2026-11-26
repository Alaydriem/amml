use std::ffi::c_void;

use amml::{ActorAdapter, BlockReadAdapter, ClientAdapter, KindsAdapter, SessionAdapter, WorldAdapter};
use amml_abi::{AmmlActorTable, AmmlApi, AmmlBlockPos, AmmlKindsResult, AmmlServices, AmmlStatus, AmmlWorldTable};

use crate::support::{ActorGame, BlockReadGame, ChunkGame, ChunkKind, ClientGame, GAME_CLIENT_A, Harness, LogFile};

// Installs the adapters of the kinds scan over every stand-in entry but `without`.
fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&BlockReadGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    BlockReadAdapter::install(harness.state);
    KindsAdapter::install(harness.state);
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
    let table = unsafe { AmmlActorTable::read(table) }.unwrap();
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

// What one `kinds_in_box` call gave: the status, the ids written and the result.
struct Answer {
    status: AmmlStatus,
    ids: Vec<u32>,
    result: AmmlKindsResult,
}

fn kinds(table: &AmmlWorldTable, source: usize, min: AmmlBlockPos, max: AmmlBlockPos, cap: u32) -> Answer {
    let mut ids = vec![u32::MAX; cap as usize];
    let mut result = AmmlKindsResult { kind: 99, _reserved: [0; 3], count: 77, cells: 77, elapsed_us: 77 };
    // SAFETY: the table is live; the positions, the ids (`cap` entries) and the result live for the call.
    let status = unsafe {
        (table.kinds_in_box.unwrap())(
            std::ptr::with_exposed_provenance(source),
            &raw const min,
            &raw const max,
            ids.as_mut_ptr(),
            cap,
            &mut result,
        )
    };
    let written = (result.count as usize).min(ids.len());
    ids.truncate(written);
    Answer { status, ids, result }
}

fn names(ids: &[u32]) -> Vec<String> {
    let world = WorldAdapter::get().unwrap();
    ids.iter().map(|id| world.block_type_name(*id).unwrap()).collect()
}

#[test]
fn the_world_table_covers_v9_with_the_scan() {
    let harness = start_in_world();

    assert!(table(&harness).size >= AmmlWorldTable::SIZE_V9);
    assert!(table(&harness).version >= 9);
    assert!(table(&harness).kinds_in_box.is_some());
    assert!(harness.state.areas().is_available("world.kinds_in_box", 1));
}

#[test]
fn kinds_are_listed_in_the_order_of_the_scan_with_air_as_a_kind() {
    let harness = start_in_world();
    ChunkGame::set_block(0, 0, 1, ChunkKind::Stone);
    ChunkGame::set_block(1, 0, 0, ChunkKind::Dirt);
    let table = table(&harness);

    // x, then z, then y: (0,0,0) air, (0,1,0) stone, (0,2,0) air, then x 1: dirt at y 0.
    let answer = kinds(&table, source_handle(&harness), pos(0, 0, 0), pos(1, 2, 0), 8);

    assert_eq!(answer.status, AmmlStatus::OK);
    assert_eq!(answer.result.kind, AmmlKindsResult::KINDS);
    assert_eq!((answer.result.count, answer.result.cells), (3, 6));
    assert_eq!(names(&answer.ids), ["minecraft:air", "minecraft:stone", "minecraft:dirt"]);
}

#[test]
fn a_chunk_that_is_not_loaded_gives_not_loaded_and_reads_no_cell() {
    let harness = start_in_world();
    ChunkGame::set_block(0, 0, 0, ChunkKind::Stone);
    let table = table(&harness);

    // The box reaches into chunk (1, 0), which the stand-in game does not have.
    let outside = kinds(&table, source_handle(&harness), pos(0, 0, 0), pos(20, 3, 3), 8);
    ChunkGame::set_load_state(0);
    let unloaded = kinds(&table, source_handle(&harness), pos(0, 0, 0), pos(3, 3, 3), 8);

    for answer in [outside, unloaded] {
        assert_eq!(answer.status, AmmlStatus::OK);
        assert_eq!((answer.result.kind, answer.result.count, answer.result.cells), (AmmlKindsResult::NOT_LOADED, 0, 0));
        assert!(answer.ids.is_empty());
    }
    assert!(ChunkGame::source_asked().is_empty());
}

#[test]
fn more_kinds_than_the_cap_gives_too_many_and_lists_nothing() {
    let harness = start_in_world();
    ChunkGame::set_block(0, 0, 1, ChunkKind::Stone);
    let table = table(&harness);

    let answer = kinds(&table, source_handle(&harness), pos(0, 0, 0), pos(0, 2, 0), 1);

    assert_eq!(answer.status, AmmlStatus::OK);
    assert_eq!((answer.result.kind, answer.result.count), (AmmlKindsResult::TOO_MANY, 0));
}

#[test]
fn box_over_the_cell_cap_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let source = source_handle(&harness);

    // 16 x 257 x 16 is 65792 cells, 256 above the cap.
    let over = kinds(&table, source, pos(0, 0, 0), pos(15, 256, 15), 8);
    assert!(ChunkGame::source_asked().is_empty());
    // 16 x 256 x 16 is exactly the cap.
    let exact = kinds(&table, source, pos(0, 0, 0), pos(15, 255, 15), 8);

    assert_eq!(over.status, AmmlStatus::REFUSED);
    assert_eq!(over.result.kind, 99);
    assert_eq!(exact.status, AmmlStatus::OK);
    assert_eq!((exact.result.kind, exact.result.cells), (AmmlKindsResult::KINDS, 65536));
}

#[test]
fn inverted_box_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let source = source_handle(&harness);

    for (min, max) in [(pos(2, 0, 0), pos(1, 5, 5)), (pos(0, 3, 0), pos(5, 2, 5)), (pos(0, 0, 9), pos(5, 5, 8))] {
        assert_eq!(kinds(&table, source, min, max, 8).status, AmmlStatus::REFUSED);
    }
    assert!(ChunkGame::source_asked().is_empty());
}

#[test]
fn bad_pointers_are_invalid_argument() {
    let harness = start_in_world();
    let table = table(&harness);
    let source: *const c_void = std::ptr::with_exposed_provenance(source_handle(&harness));
    let (min, max) = (pos(0, 0, 0), pos(1, 1, 1));
    let mut ids = [0u32; 4];
    let mut result = AmmlKindsResult::default();
    let call = table.kinds_in_box.unwrap();

    // SAFETY: the table is live; each call has a null among otherwise live arguments, which the loader checks first.
    let statuses = unsafe {
        [
            call(source, std::ptr::null(), &raw const max, ids.as_mut_ptr(), 4, &mut result),
            call(source, &raw const min, std::ptr::null(), ids.as_mut_ptr(), 4, &mut result),
            call(source, &raw const min, &raw const max, std::ptr::null_mut(), 4, &mut result),
            call(source, &raw const min, &raw const max, ids.as_mut_ptr(), 4, std::ptr::null_mut()),
        ]
    };

    assert_eq!(statuses, [AmmlStatus::INVALID_ARGUMENT; 4]);
}

#[test]
fn a_block_source_of_another_world_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let before = source_handle(&harness);

    ClientGame::frame();
    let stale = kinds(&table, before ^ (1 << 48), pos(0, 0, 0), pos(1, 1, 1), 8);

    assert_eq!(stale.status, AmmlStatus::REFUSED);
}

#[test]
fn off_the_game_thread_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let source = source_handle(&harness);

    let status = std::thread::scope(|scope| {
        scope.spawn(|| kinds(&table, source, pos(0, 0, 0), pos(1, 1, 1), 8).status).join().unwrap()
    });

    assert_eq!(status, AmmlStatus::REFUSED);
    assert!(ChunkGame::source_asked().is_empty());
}

#[test]
fn the_area_is_not_offered_without_get_block() {
    let harness = install(&["BlockSource.getBlock"]);
    join(&harness);

    assert!(!harness.state.areas().is_available("world.kinds_in_box", 1));
}

#[test]
fn the_scan_time_is_logged_for_the_first_call_only() {
    let harness = start_in_world();
    ChunkGame::set_block(0, 0, 1, ChunkKind::Stone);
    let table = table(&harness);
    let source = source_handle(&harness);

    let first = kinds(&table, source, pos(0, 0, 0), pos(0, 2, 0), 8);
    let second = kinds(&table, source, pos(0, 0, 0), pos(0, 2, 0), 8);

    assert_eq!((first.status, second.status), (AmmlStatus::OK, AmmlStatus::OK));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "kinds scan time");
    let scans = log.lines().filter(|line| line.contains("kinds scan time")).count();
    assert_eq!(scans, 1, "{log}");
    assert!(log.lines().any(|line| line.contains("kinds scan time") && line.contains("cells")), "{log}");
}
