use std::ffi::c_void;

use amml::{
    ActorAdapter, CameraAdapter, ClientAdapter, FreeCameraAdapter, HandleStamp, ModLoader, SessionAdapter,
    ShapesAdapter, SurfaceShape, WorldAdapter,
};
use amml_abi::{
    AmmlAabb, AmmlActorBox, AmmlActorTable, AmmlApi, AmmlApplyStats, AmmlBlockHit, AmmlBlockPos, AmmlChunkLoad,
    AmmlColumnReadStats, AmmlColumnRequest, AmmlColumnSample, AmmlFace, AmmlRayOptions, AmmlRgba, AmmlServices,
    AmmlShapeOp, AmmlStatus, AmmlStr, AmmlSurfaceSample, AmmlVec3, AmmlWorldTable,
};
use amml_runtime::ModRegistry;

use crate::support::{
    ActorGame, CLIENT_A, Candidate, ChunkGame, ChunkKind, ClientGame, ColorGame, ColorShape, EventsMod, FreeCameraGame,
    GAME_CLIENT_A, Harness, HeapProbe, LogFile, RayGame, SessionGame, SpanOrder, SurfaceGame,
};

// Installs the session, client, actor and world adapters over every stand-in entry but `without`, for the chunk
// layout whose `mSubChunks` is at `sub_chunks` and the material layout `material`.
fn install_with(without: &[&str], sub_chunks: u64, material: u64) -> Harness {
    let harness = Harness::start_with_entries(&ChunkGame::entries(without, sub_chunks, material));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    harness
}

fn install(without: &[&str]) -> Harness {
    install_with(without, 0x1C8, 1)
}

// After one frame and a joined world whose local player is the stand-in player.
fn join(harness: &Harness) {
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(ActorGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 1);
}

// `install` with every entry, joined.
fn start_in_world() -> Harness {
    let harness = install(&[]);
    join(&harness);
    harness
}

fn services(harness: &Harness) -> *const AmmlServices {
    // SAFETY: the loader's tables live for the process.
    unsafe { AmmlApi::services(harness.state.api()) }.unwrap()
}

// The world table, through the services table as a mod reaches it.
fn table(harness: &Harness) -> AmmlWorldTable {
    // SAFETY: the loader's tables live for the process.
    let table = unsafe { AmmlServices::world(services(harness)) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlWorldTable::read(table) }.unwrap()
}

// The block source handle the actor table gives for the stand-in player.
fn block_source_handle(harness: &Harness) -> usize {
    // SAFETY: the loader's tables live for the process.
    let table = unsafe { AmmlServices::actor(services(harness)) }.unwrap();
    // SAFETY: as above.
    let table: AmmlActorTable = unsafe { AmmlActorTable::read(table) }.unwrap();
    let mut out: *const c_void = std::ptr::null();
    // SAFETY: the table is live and `out` is writable.
    let status =
        unsafe { (table.block_source.unwrap())(std::ptr::with_exposed_provenance(ActorGame::player()), &mut out) };
    assert_eq!(status, AmmlStatus::OK);
    out.addr()
}

fn sky_darken(table: &AmmlWorldTable, dimension: usize) -> (AmmlStatus, u8) {
    let mut out = 0xEEu8;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.sky_darken)(std::ptr::with_exposed_provenance::<c_void>(dimension), &mut out) };
    (status, out)
}

// `table.chunk` for the chunk (`x`, `z`) of `source`; the handle is `usize::MAX` while `out` was not written.
fn chunk(table: &AmmlWorldTable, source: usize, x: i32, z: i32) -> (AmmlStatus, usize) {
    let mut out = std::ptr::with_exposed_provenance::<c_void>(usize::MAX);
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.chunk.unwrap())(std::ptr::with_exposed_provenance(source), x, z, &mut out) };
    (status, out.addr())
}

fn load_state(table: &AmmlWorldTable, chunk: usize) -> (AmmlStatus, AmmlChunkLoad) {
    let mut out = AmmlChunkLoad(0xEE);
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.chunk_load_state.unwrap())(std::ptr::with_exposed_provenance(chunk), &mut out) };
    (status, out)
}

fn request(camera_y: i32, flags: u8, columns: &[(usize, usize)]) -> AmmlColumnRequest {
    let mut words = [0u64; 4];
    for (x, z) in columns {
        let bit = z * 16 + x;
        words[bit / 64] |= 1 << (bit % 64);
    }
    AmmlColumnRequest { camera_y, flags, _reserved: [0; 3], columns: words }
}

const UNTOUCHED: u32 = 0xDEAD;

// Samples that a read leaves as they are for a column it was not asked for.
fn untouched() -> Vec<AmmlColumnSample> {
    let sample = AmmlColumnSample {
        shown: UNTOUCHED,
        water: 0,
        shown_color: AmmlRgba { r: 0, g: 0, b: 0, a: 0 },
        water_color: AmmlRgba { r: 0, g: 0, b: 0, a: 0 },
        shown_y: 0,
        height: 0,
        solid: 0,
        biome: 0,
        water_depth: 0,
        flags: 0,
        _reserved: [0; 2],
    };
    vec![sample; 256]
}

fn read_columns(
    table: &AmmlWorldTable,
    chunk: usize,
    request: &AmmlColumnRequest,
    samples: &mut [AmmlColumnSample],
) -> (AmmlStatus, AmmlColumnReadStats) {
    let mut stats = AmmlColumnReadStats::default();
    // SAFETY: the table is live; `request`, the samples and `stats` are writable for the call.
    let status = unsafe {
        (table.read_columns.unwrap())(
            std::ptr::with_exposed_provenance(chunk),
            request,
            samples.as_mut_ptr(),
            samples.len() as u32,
            &mut stats,
        )
    };
    (status, stats)
}

fn type_name(table: &AmmlWorldTable, id: u32, cap: usize) -> (AmmlStatus, String, usize) {
    let mut buffer = vec![0u8; cap];
    let mut length = usize::MAX;
    // SAFETY: the table is live; `buffer` holds `cap` bytes and `length` is writable.
    let status = unsafe { (table.block_type_name.unwrap())(id, buffer.as_mut_ptr(), cap, &mut length) };
    let copied = length.min(cap);
    (status, String::from_utf8_lossy(&buffer[..copied]).into_owned(), length)
}

// The handle of the chunk (0, 0) of the player's block source.
fn chunk_handle(harness: &Harness) -> usize {
    let (status, handle) = chunk(&table(harness), block_source_handle(harness), 0, 0);
    assert_eq!(status, AmmlStatus::OK);
    assert_ne!(handle, 0);
    handle
}

#[test]
fn the_services_table_has_a_v9_world_table() {
    let harness = Harness::start();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { &*services(&harness) };
    assert!(services.size >= AmmlServices::SIZE_V9);
    assert_eq!(table(&harness).size, AmmlWorldTable::SIZE_V9);
    assert_eq!(table(&harness).version, 9);
}

#[test]
fn null_out_is_invalid_argument() {
    let harness = start_in_world();
    // SAFETY: the table is live; the loader checks `out` before it writes.
    let status = unsafe { (table(&harness).sky_darken)(std::ptr::null(), std::ptr::null_mut()) };
    assert_eq!(status, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn no_adapter_is_not_available() {
    let harness = Harness::start();
    assert_eq!(sky_darken(&table(&harness), ActorGame::dimension()).0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn before_the_gate_is_not_ready() {
    let harness = install(&[]);
    assert_eq!(sky_darken(&table(&harness), ActorGame::dimension()).0, AmmlStatus::NOT_READY);
}

#[test]
fn the_area_is_not_offered_without_the_sky_darken_entry() {
    let harness = install(&["Dimension.mSkyDarken"]);
    assert!(!harness.state.areas().is_available(WorldAdapter::SKY_DARKEN, 1));
    ClientGame::frame();
    ClientGame::set_player(ActorGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(sky_darken(&table(&harness), ActorGame::dimension()).0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn the_sky_darkness_is_the_byte_of_the_dimension_of_the_player() {
    let harness = start_in_world();
    let table = table(&harness);
    assert_eq!(sky_darken(&table, ActorGame::dimension()), (AmmlStatus::OK, 4));
    ActorGame::set_sky_darken(11);
    assert_eq!(sky_darken(&table, ActorGame::dimension()), (AmmlStatus::OK, 11));
}

#[test]
fn a_dimension_that_is_not_the_one_of_the_player_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    assert_eq!(sky_darken(&table, ActorGame::dimension() + 8).0, AmmlStatus::REFUSED);
    assert_eq!(sky_darken(&table, 0).0, AmmlStatus::REFUSED);
}

#[test]
fn a_call_off_the_game_thread_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let status =
        std::thread::scope(|scope| scope.spawn(|| sky_darken(&table, ActorGame::dimension()).0).join().unwrap());
    assert_eq!(status, AmmlStatus::REFUSED);
}

fn block_source_check(handle: usize) -> Result<usize, AmmlStatus> {
    let handle = std::ptr::with_exposed_provenance::<c_void>(handle);
    WorldAdapter::get().unwrap().block_source(WorldAdapter::SKY_DARKEN, 1, handle)
}

#[test]
fn the_block_source_of_the_dimension_of_the_player_is_accepted() {
    let harness = start_in_world();
    assert_eq!(block_source_check(block_source_handle(&harness)), Ok(ActorGame::block_source()));
}

#[test]
fn a_block_source_of_another_dimension_is_refused() {
    let harness = start_in_world();
    let other = HandleStamp::seal(ActorGame::other_block_source(), 1).unwrap();
    assert_eq!(block_source_check(other), Err(AmmlStatus::REFUSED));
    assert_eq!(block_source_check(0), Err(AmmlStatus::REFUSED));
    // The address without the epoch of the world is no handle the loader gave.
    assert_eq!(block_source_check(ActorGame::block_source()), Err(AmmlStatus::REFUSED));
    assert_eq!(block_source_check(block_source_handle(&harness)), Ok(ActorGame::block_source()));
}

#[test]
fn a_block_source_of_another_dimension_is_refused_through_the_table() {
    let harness = start_in_world();
    let table = table(&harness);
    let other = HandleStamp::seal(ActorGame::other_block_source(), 1).unwrap();

    assert_eq!(chunk(&table, other, 0, 0), (AmmlStatus::REFUSED, usize::MAX));
    assert_eq!(chunk(&table, 0, 0, 0), (AmmlStatus::REFUSED, usize::MAX));
}

#[test]
fn a_block_source_is_refused_after_the_player_left() {
    let harness = start_in_world();
    let handle = block_source_handle(&harness);
    ClientGame::set_player(0);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(block_source_check(handle), Err(AmmlStatus::REFUSED));
}

#[test]
fn a_block_source_from_before_a_world_leave_is_refused_at_the_same_address() {
    let harness = start_in_world();
    let before = block_source_handle(&harness);
    SessionGame::exit();
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 0);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 2);

    // The stand-in gives the same block source address again, as a game that reuses a freed address.
    let after = block_source_handle(&harness);

    assert_eq!(HandleStamp::open(before, 1), Some(ActorGame::block_source()));
    assert_eq!(block_source_check(before), Err(AmmlStatus::REFUSED));
    assert_eq!(chunk(&table(&harness), before, 0, 0), (AmmlStatus::REFUSED, usize::MAX));
    assert_eq!(block_source_check(after), Ok(ActorGame::block_source()));
}

#[test]
fn the_chunk_of_a_loaded_position_is_a_handle_and_another_position_gives_none() {
    let harness = start_in_world();
    let table = table(&harness);
    let source = block_source_handle(&harness);

    let (status, handle) = chunk(&table, source, 0, 0);
    let none = chunk(&table, source, 1, 0);

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(handle & ADDRESS_MASK, ChunkGame::chunk());
    assert_ne!(handle >> 48, 0);
    assert_eq!(none, (AmmlStatus::OK, 0));
}

// The low 48 bits of a handle: the address of the game object.
const ADDRESS_MASK: usize = (1 << 48) - 1;

#[test]
fn a_chunk_handle_from_an_older_frame_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let old = chunk_handle(&harness);
    assert_eq!(load_state(&table, old), (AmmlStatus::OK, AmmlChunkLoad::LOADED));

    ClientGame::frame();

    let mut samples = untouched();
    assert_eq!(load_state(&table, old).0, AmmlStatus::REFUSED);
    assert_eq!(read_columns(&table, old, &request(0, 0, &[(0, 0)]), &mut samples).0, AmmlStatus::REFUSED);
    assert_eq!(samples[0].shown, UNTOUCHED);
    // A handle of this frame reads.
    assert_eq!(load_state(&table, chunk_handle(&harness)), (AmmlStatus::OK, AmmlChunkLoad::LOADED));
}

#[test]
fn a_chunk_handle_that_was_never_given_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    // A handle with the stamp of this frame, for an address the loader did not give.
    let stamp = chunk_handle(&harness) >> 48;
    let stamped = (stamp << 48) | (ChunkGame::chunk() + 0x40);

    assert_eq!(load_state(&table, stamped).0, AmmlStatus::REFUSED);
    assert_eq!(load_state(&table, ChunkGame::chunk()).0, AmmlStatus::REFUSED);
    assert_eq!(load_state(&table, 0).0, AmmlStatus::REFUSED);
}

#[test]
fn read_columns_len_other_than_256_is_invalid_argument() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    let all = request(0, 0, &[(0, 0)]);

    let mut short = untouched();
    short.truncate(255);
    let mut long = untouched();
    long.push(long[0]);

    assert_eq!(read_columns(&table, handle, &all, &mut short).0, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(read_columns(&table, handle, &all, &mut long).0, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(short[0].shown, UNTOUCHED);
}

#[test]
fn placeholder_top_sub_chunk_gives_top_placeholder() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ChunkGame::set_placeholder(ChunkGame::SUB_CHUNKS - 1, true);

    let mut samples = untouched();
    let (status, stats) = read_columns(&table, handle, &request(0, 0, &[(1, 1), (2, 2)]), &mut samples);

    assert_eq!(load_state(&table, handle), (AmmlStatus::OK, AmmlChunkLoad::TOP_PLACEHOLDER));
    assert_eq!(status, AmmlStatus::OK);
    assert_eq!((stats.read, stats.skipped, stats.retry), (0, 2, 0));
    assert_eq!(samples[17].flags, AmmlColumnSample::SKIPPED);
    assert_eq!(samples[34].flags, AmmlColumnSample::SKIPPED);
    assert_eq!(samples[0].shown, UNTOUCHED);

    ChunkGame::set_placeholder(ChunkGame::SUB_CHUNKS - 1, false);
    assert_eq!(load_state(&table, handle), (AmmlStatus::OK, AmmlChunkLoad::LOADED));
}

#[test]
fn a_chunk_that_is_not_loaded_reads_not_loaded_and_skips_its_columns() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ChunkGame::set_load_state(5);

    let mut samples = untouched();
    let (_, stats) = read_columns(&table, handle, &request(0, 0, &[(0, 0)]), &mut samples);

    assert_eq!(load_state(&table, handle), (AmmlStatus::OK, AmmlChunkLoad::NOT_LOADED));
    assert_eq!((stats.read, stats.skipped), (0, 1));
    assert_eq!(samples[0].flags, AmmlColumnSample::SKIPPED);
}

#[test]
fn a_column_whose_surface_sub_chunk_is_a_placeholder_is_skipped_and_the_others_are_read() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    // Height 10 is raw 74, in sub-chunk 4; height -50 is raw 14, in sub-chunk 0.
    ChunkGame::fill(1, 0, -64, 10, ChunkKind::Stone);
    ChunkGame::set_height(1, 0, 10);
    ChunkGame::fill(2, 0, -64, -50, ChunkKind::Stone);
    ChunkGame::set_height(2, 0, -50);
    ChunkGame::set_placeholder(4, true);

    let mut samples = untouched();
    let (_, stats) = read_columns(&table, handle, &request(30, 0, &[(1, 0), (2, 0)]), &mut samples);

    assert_eq!((stats.read, stats.skipped), (1, 1));
    assert_eq!(samples[1].flags, AmmlColumnSample::SKIPPED);
    assert_eq!(samples[2].shown_y, -50);
}

#[test]
fn only_the_asked_columns_are_written_and_the_walk_finds_the_surface() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ChunkGame::fill(3, 2, -64, 10, ChunkKind::Stone);
    ChunkGame::set_height(3, 2, 10);

    let mut samples = untouched();
    let (status, stats) = read_columns(&table, handle, &request(30, 0, &[(3, 2)]), &mut samples);

    let at = 2 * 16 + 3;
    assert_eq!(status, AmmlStatus::OK);
    assert_eq!((stats.read, stats.skipped, stats.retry), (1, 0, 0));
    assert_eq!((samples[at].shown_y, samples[at].height, samples[at].solid), (10, 10, 10));
    assert_ne!(samples[at].shown, 0);
    assert_eq!(samples[at].flags, AmmlColumnSample::NO_COLOR);
    assert!(samples.iter().enumerate().all(|(index, sample)| index == at || sample.shown == UNTOUCHED));
    let (status, name, _) = type_name(&table, samples[at].shown, 64);
    assert_eq!((status, name.as_str()), (AmmlStatus::OK, "minecraft:stone"));
    // Every block the walk asked for is inside the chunk.
    let asked = ChunkGame::asked();
    assert!(!asked.is_empty() && asked.iter().all(|(_, _, y)| (0..128).contains(y)), "{asked:?}");
}

#[test]
fn transparent_water_and_a_placeholder_block_set_their_flags() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ChunkGame::fill(0, 0, -64, -3, ChunkKind::Stone);
    ChunkGame::fill(0, 0, -2, 0, ChunkKind::Water);
    ChunkGame::set_height(0, 0, 0);
    ChunkGame::fill(1, 0, -64, 0, ChunkKind::Stone);
    ChunkGame::set_block(1, 0, 0, ChunkKind::Placeholder);
    ChunkGame::set_height(1, 0, 0);

    let mut samples = untouched();
    let (_, stats) = read_columns(
        &table,
        handle,
        &request(20, AmmlColumnRequest::TRANSPARENT_WATER, &[(0, 0), (1, 0)]),
        &mut samples,
    );

    assert_eq!((samples[0].shown_y, samples[0].water_depth), (-3, 3));
    assert_ne!(samples[0].water, 0);
    assert_ne!(samples[0].water, samples[0].shown);
    assert_eq!(samples[1].flags & AmmlColumnSample::RETRY, AmmlColumnSample::RETRY);
    assert_eq!((stats.read, stats.retry), (2, 1));
}

#[test]
fn a_column_of_air_is_void() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);

    let mut samples = untouched();
    read_columns(&table, handle, &request(20, 0, &[(5, 5)]), &mut samples);

    assert_eq!(samples[5 * 16 + 5].flags, AmmlColumnSample::VOID);
    assert_eq!(samples[5 * 16 + 5].shown, 0);
}

#[test]
fn the_chunk_reads_work_on_the_other_layout_and_the_pointer_material() {
    // 70.26: `mSubChunks` at 228h; the Material held as a pointer, so the solid flag is the bool byte at 5.
    let harness = install_with(&[], 0x228, 0);
    join(&harness);
    let table = table(&harness);
    ChunkGame::fill(4, 4, -64, -10, ChunkKind::Stone);
    ChunkGame::fill(4, 4, -9, -5, ChunkKind::Water);
    ChunkGame::fill(4, 4, -4, 7, ChunkKind::Stone);
    ChunkGame::fill(4, 4, 8, 9, ChunkKind::Water);
    ChunkGame::set_height(4, 4, 9);
    let handle = chunk_handle(&harness);

    let mut samples = untouched();
    // The camera is inside the water, which is not solid: it is shown.
    let (_, stats) = read_columns(&table, handle, &request(8, 0, &[(4, 4)]), &mut samples);

    let at = 4 * 16 + 4;
    assert_eq!(load_state(&table, handle), (AmmlStatus::OK, AmmlChunkLoad::LOADED));
    assert_eq!((stats.read, samples[at].shown_y, samples[at].height), (1, 8, 9));
    assert_eq!(type_name(&table, samples[at].shown, 64).1, "minecraft:water");

    // The camera is inside the stone, which is solid: cave mode shows the first block under it that is not solid.
    read_columns(&table, handle, &request(5, 0, &[(4, 4)]), &mut samples);
    assert_eq!(samples[at].shown_y, -5);
    assert_eq!(type_name(&table, samples[at].shown, 64).1, "minecraft:water");
}

#[test]
fn block_type_name_is_cut_to_the_buffer_and_gives_the_full_length() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ChunkGame::fill(0, 0, -64, 0, ChunkKind::Stone);
    ChunkGame::set_height(0, 0, 0);
    let mut samples = untouched();
    read_columns(&table, handle, &request(10, 0, &[(0, 0)]), &mut samples);
    let id = samples[0].shown;

    assert_eq!(type_name(&table, id, 9), (AmmlStatus::OK, "minecraft".to_string(), 15));
    assert_eq!(type_name(&table, id, 0), (AmmlStatus::OK, String::new(), 15));
    assert_eq!(type_name(&table, 0, 64).0, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(type_name(&table, id + 100, 64).0, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn block_type_name_works_off_the_game_thread() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ChunkGame::fill(0, 0, -64, 0, ChunkKind::Stone);
    ChunkGame::set_height(0, 0, 0);
    let mut samples = untouched();
    read_columns(&table, handle, &request(10, 0, &[(0, 0)]), &mut samples);
    let id = samples[0].shown;

    let name = std::thread::scope(|scope| scope.spawn(|| type_name(&table, id, 64)).join().unwrap());

    assert_eq!((name.0, name.1.as_str()), (AmmlStatus::OK, "minecraft:stone"));
}

#[test]
fn the_chunk_calls_off_the_game_thread_are_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let source = block_source_handle(&harness);
    let handle = chunk_handle(&harness);

    let (chunk_status, state_status) = std::thread::scope(|scope| {
        scope.spawn(|| (chunk(&table, source, 0, 0).0, load_state(&table, handle).0)).join().unwrap()
    });

    assert_eq!((chunk_status, state_status), (AmmlStatus::REFUSED, AmmlStatus::REFUSED));
}

#[test]
fn null_pointers_of_the_chunk_calls_are_invalid_arguments() {
    let harness = start_in_world();
    let table = table(&harness);
    let all = request(0, 0, &[(0, 0)]);
    let mut samples = untouched();
    let mut stats = AmmlColumnReadStats::default();

    // SAFETY: the table is live; the loader checks every pointer before it uses it.
    let statuses = unsafe {
        [
            (table.chunk.unwrap())(std::ptr::null(), 0, 0, std::ptr::null_mut()),
            (table.chunk_load_state.unwrap())(std::ptr::null(), std::ptr::null_mut()),
            (table.read_columns.unwrap())(std::ptr::null(), std::ptr::null(), samples.as_mut_ptr(), 256, &mut stats),
            (table.read_columns.unwrap())(std::ptr::null(), &all, std::ptr::null_mut(), 256, &mut stats),
            (table.block_type_name.unwrap())(1, std::ptr::null_mut(), 0, std::ptr::null_mut()),
        ]
    };

    assert_eq!(statuses, [AmmlStatus::INVALID_ARGUMENT; 5]);
}

#[test]
fn an_unknown_sub_chunks_value_offers_no_chunk_area_and_logs_it() {
    let harness = install_with(&[], 0x140, 1);
    join(&harness);

    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "chunk layout unknown");

    assert!(log.contains("chunk layout unknown"), "{log}");
    assert!(!harness.state.areas().is_available(WorldAdapter::CHUNK_READ, 1));
    assert!(harness.state.areas().is_available(WorldAdapter::SKY_DARKEN, 1));
    let source = block_source_handle(&harness);
    assert_eq!(chunk(&table(&harness), source, 0, 0).0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn the_chunk_area_is_not_offered_without_one_of_its_entries() {
    let harness = install(&["Block.mBlockType"]);
    assert!(!harness.state.areas().is_available(WorldAdapter::CHUNK_READ, 1));
    assert!(harness.state.areas().is_available(WorldAdapter::SKY_DARKEN, 1));
}

// The loader log once a `world thread` line for `hook` is in it; the log is written by a background thread.
fn wait_for_thread_line(harness: &Harness, hook: &str) -> String {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let log = LogFile::wait_for(&harness.logs().join("loader.log"), "world thread");
        let found = log.lines().any(|line| line.contains("world thread") && line.contains(hook));
        if found || std::time::Instant::now() > deadline {
            return log;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn the_thread_of_each_world_call_is_logged_once_and_again_when_it_changes() {
    let harness = start_in_world();
    let table = table(&harness);
    let dimension = ActorGame::dimension();
    for _ in 0..3 {
        sky_darken(&table, dimension);
    }
    std::thread::scope(|scope| scope.spawn(|| sky_darken(&table, dimension)).join().unwrap());
    let source = block_source_handle(&harness);
    chunk(&table, source, 0, 0);

    let log = wait_for_thread_line(&harness, "BlockSource.getChunk");

    let lines = |hook: &str| log.lines().filter(|line| line.contains("world thread") && line.contains(hook)).count();
    assert_eq!(lines("Dimension.mSkyDarken"), 2, "{log}");
    assert_eq!(lines("BlockSource.getChunk"), 1, "{log}");
}

#[test]
fn the_top_layer_at_mmax_y_is_read_and_nothing_above_it() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    let top = ChunkGame::MAX_Y - 1;
    ChunkGame::fill(0, 0, -64, top, ChunkKind::Stone);
    ChunkGame::set_height(0, 0, top);

    let mut samples = untouched();
    read_columns(&table, handle, &request(500, 0, &[(0, 0), (1, 0)]), &mut samples);

    assert_eq!(samples[0].shown_y, i16::try_from(top).unwrap());
    let asked = ChunkGame::asked();
    assert!(asked.contains(&(0, 0, 127)), "{asked:?}");
    assert!(asked.iter().all(|(_, _, y)| (0..128).contains(y)), "{asked:?}");
}

// Installs the adapters over the chunk stand-ins and the colour stand-ins of `shape`, minus `without`, and joins.
fn start_with_colors(shape: ColorShape, without: &[&str]) -> Harness {
    let mut entries = ChunkGame::entries(without, 0x1C8, 1);
    entries.extend(ColorGame::entries(shape, without));
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    join(&harness);
    harness
}

const COLORS: u8 = AmmlColumnRequest::VANILLA_COLORS;
const NO_PAINT: AmmlRgba = AmmlRgba { r: 0, g: 0, b: 0, a: 0 };

fn biome_name(table: &AmmlWorldTable, id: u16, cap: usize) -> (AmmlStatus, String, usize) {
    let mut buffer = vec![0u8; cap];
    let mut length = usize::MAX;
    // SAFETY: the table is live; `buffer` holds `cap` bytes and `length` is writable.
    let status = unsafe { (table.biome_name.unwrap())(id, buffer.as_mut_ptr(), cap, &mut length) };
    let copied = length.min(cap);
    (status, String::from_utf8_lossy(&buffer[..copied]).into_owned(), length)
}

fn stone_to(x: u8, z: u8, top: i32) {
    ChunkGame::fill(x, z, -64, top, ChunkKind::Stone);
    ChunkGame::set_height(usize::from(x), usize::from(z), top);
}

#[test]
fn colors_without_the_area_set_no_color() {
    let harness = start_in_world();
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    stone_to(0, 0, 10);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, COLORS, &[(0, 0)]), &mut samples);

    assert!(!harness.state.areas().is_available("world.map_color", 1));
    assert!(!harness.state.areas().is_available("world.biomes", 1));
    assert_eq!(samples[0].flags, AmmlColumnSample::NO_COLOR);
    assert_eq!((samples[0].shown_color, samples[0].water_color), (NO_PAINT, NO_PAINT));
    assert_eq!(samples[0].biome, AmmlColumnSample::NO_BIOME);
    assert_eq!(biome_name(&table, 1, 16).0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn the_block_shape_gives_the_map_colour_of_the_shown_block_and_the_biome_id() {
    let harness = start_with_colors(ColorShape::Block, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    stone_to(0, 0, 10);
    stone_to(0, 9, 10);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, COLORS, &[(0, 0), (0, 9)]), &mut samples);

    let stone = AmmlRgba { r: 128, g: 64, b: 255, a: 255 };
    assert_eq!((samples[0].shown_color, samples[0].flags), (stone, 0));
    assert_eq!((samples[0].biome, samples[9 * 16].biome), (ColorGame::PLAINS_ID, ColorGame::SWAMP_ID));
    // The colour is read through the main chunk source, at the world position of the shown block.
    let source = ActorGame::block_source();
    assert_eq!(
        ColorGame::color_calls(),
        [(source, [ColorGame::MIN_X, 10, ColorGame::MIN_Z]), (source, [ColorGame::MIN_X, 10, ColorGame::MIN_Z + 9])]
    );
}

#[test]
fn the_component_shape_tints_the_component_colour_by_its_biome_policy() {
    let harness = start_with_colors(ColorShape::Component, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    stone_to(3, 0, 10);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, COLORS, &[(3, 0)]), &mut samples);

    // The grey component 0.5 times the tint 0.5, 1, 0.
    assert_eq!(samples[3].shown_color, AmmlRgba { r: 64, g: 128, b: 0, a: 255 });
    assert_eq!(ColorGame::tints(), [3]);
    assert_eq!(ColorGame::color_calls(), [(ActorGame::block_source(), [ColorGame::MIN_X + 3, 10, ColorGame::MIN_Z])]);
}

#[test]
fn water_over_a_block_gives_the_biome_water_colour_at_the_water_surface() {
    let harness = start_with_colors(ColorShape::Block, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ChunkGame::fill(1, 1, -64, -3, ChunkKind::Stone);
    ChunkGame::fill(1, 1, -2, 0, ChunkKind::Water);
    ChunkGame::set_height(1, 1, 0);

    let mut samples = untouched();
    let flags = COLORS | AmmlColumnRequest::TRANSPARENT_WATER;
    read_columns(&table, handle, &request(30, flags, &[(1, 1)]), &mut samples);

    let at = 16 + 1;
    assert_eq!(samples[at].water_color, AmmlRgba { r: 0x10, g: 0x20, b: 0x31, a: 255 });
    assert_eq!(samples[at].shown_color, AmmlRgba { r: 128, g: 64, b: 255, a: 255 });
    let water: Vec<_> = ColorGame::sampled().into_iter().filter(|call| call.entry == "water").collect();
    assert_eq!(water.len(), 1);
    assert_eq!((water[0].biome, water[0].pos), (ColorGame::PLAINS_ID, [ColorGame::MIN_X + 1, 0, ColorGame::MIN_Z + 1]));
}

#[test]
fn a_shown_water_block_has_the_biome_water_colour() {
    let harness = start_with_colors(ColorShape::Block, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ChunkGame::fill(2, 9, -64, -3, ChunkKind::Stone);
    ChunkGame::fill(2, 9, -2, 0, ChunkKind::Water);
    ChunkGame::set_height(2, 9, 0);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, COLORS, &[(2, 9)]), &mut samples);

    let at = 9 * 16 + 2;
    assert_eq!(samples[at].shown_color, AmmlRgba { r: 0x10, g: 0x20, b: 0x30 + 6, a: 255 });
    assert!(ColorGame::color_calls().is_empty());
}

#[test]
fn the_component_shape_asks_the_block_type_for_a_block_with_no_component() {
    let harness = start_with_colors(ColorShape::Component, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ChunkGame::fill(1, 0, -64, 10, ChunkKind::Dirt);
    ChunkGame::set_height(1, 0, 10);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, COLORS, &[(1, 0)]), &mut samples);

    // The colour of the type, with no tint: 0.25, 0.5, 0.75. The type is the receiver, the block the fifth argument.
    assert_eq!((samples[1].shown_color, samples[1].flags), (AmmlRgba { r: 64, g: 128, b: 191, a: 255 }, 0));
    assert_eq!(
        ColorGame::type_color_calls(),
        [(
            ChunkGame::block_type(ChunkKind::Dirt),
            ChunkGame::block(ChunkKind::Dirt),
            ActorGame::block_source(),
            [ColorGame::MIN_X + 1, 10, ColorGame::MIN_Z],
        )]
    );
    assert!(ColorGame::tints().is_empty());
}

#[test]
fn a_void_column_that_was_asked_for_a_colour_and_has_none_sets_no_color() {
    let harness = start_with_colors(ColorShape::Block, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    ColorGame::set_no_biome(true);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, COLORS, &[(4, 4)]), &mut samples);

    let at = 4 * 16 + 4;
    assert_eq!(samples[at].flags, AmmlColumnSample::VOID | AmmlColumnSample::NO_COLOR);
    assert_eq!(samples[at].shown_color, NO_PAINT);
}

#[test]
fn a_column_with_no_block_has_the_map_grass_colour_of_its_biome() {
    let harness = start_with_colors(ColorShape::Block, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, COLORS, &[(4, 4)]), &mut samples);

    let at = 4 * 16 + 4;
    assert_eq!(samples[at].flags, AmmlColumnSample::VOID);
    assert_eq!(samples[at].shown_color, AmmlRgba { r: 0x40, g: 0x50, b: 0x61, a: 255 });
    let grass = ColorGame::sampled();
    assert_eq!(grass.len(), 1);
    assert_eq!(grass[0].pos, [ColorGame::MIN_X + 4, ChunkGame::MIN_Y, ColorGame::MIN_Z + 4]);
}

#[test]
fn a_request_without_vanilla_colors_gives_no_color_and_still_the_biome() {
    let harness = start_with_colors(ColorShape::Block, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    stone_to(0, 0, 10);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, 0, &[(0, 0)]), &mut samples);

    assert_eq!(samples[0].flags, AmmlColumnSample::NO_COLOR);
    assert_eq!((samples[0].shown_color, samples[0].biome), (NO_PAINT, ColorGame::PLAINS_ID));
    assert!(ColorGame::color_calls().is_empty());
}

#[test]
fn the_map_color_area_needs_a_complete_shape_and_the_other_areas_stay() {
    let harness = start_with_colors(ColorShape::Component, &["MapPolicy.get"]);

    let areas = harness.state.areas();

    assert!(!areas.is_available("world.map_color", 1));
    assert!(areas.is_available("world.biomes", 1));
    assert!(areas.is_available("world.chunk_read", 1));
}

#[test]
fn the_biomes_area_needs_the_biome_entries_and_the_colours_stay() {
    let harness = start_with_colors(ColorShape::Block, &["Biome.mHash"]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    stone_to(0, 0, 10);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, COLORS, &[(0, 0)]), &mut samples);

    assert!(!harness.state.areas().is_available("world.biomes", 1));
    assert!(harness.state.areas().is_available("world.map_color", 1));
    assert_eq!((samples[0].biome, samples[0].flags), (AmmlColumnSample::NO_BIOME, 0));
}

#[test]
fn a_biome_name_is_known_after_a_read_met_its_id_and_is_cut_to_the_buffer() {
    let harness = start_with_colors(ColorShape::Block, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    assert_eq!(biome_name(&table, ColorGame::SWAMP_ID, 16).0, AmmlStatus::INVALID_ARGUMENT);
    stone_to(0, 9, 10);

    let mut samples = untouched();
    read_columns(&table, handle, &request(30, 0, &[(0, 9)]), &mut samples);

    assert_eq!(biome_name(&table, ColorGame::SWAMP_ID, 16), (AmmlStatus::OK, "swamp".to_string(), 5));
    assert_eq!(biome_name(&table, ColorGame::SWAMP_ID, 3), (AmmlStatus::OK, "swa".to_string(), 5));
    assert_eq!(biome_name(&table, ColorGame::PLAINS_ID, 16).0, AmmlStatus::INVALID_ARGUMENT);
    let off_thread =
        std::thread::scope(|scope| scope.spawn(|| biome_name(&table, ColorGame::SWAMP_ID, 16)).join().unwrap());
    assert_eq!((off_thread.0, off_thread.1.as_str()), (AmmlStatus::OK, "swamp"));
}

#[test]
fn a_new_world_epoch_reads_the_name_of_a_biome_id_again() {
    let harness = start_with_colors(ColorShape::Block, &[]);
    let table = table(&harness);
    let handle = chunk_handle(&harness);
    stone_to(0, 9, 10);
    let mut samples = untouched();
    read_columns(&table, handle, &request(30, 0, &[(0, 9)]), &mut samples);
    assert_eq!(biome_name(&table, ColorGame::SWAMP_ID, 16).1, "swamp");

    SessionGame::exit();
    ClientGame::update(&GAME_CLIENT_A);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 2);
    // The id is the same, but this world means another biome by it.
    ColorGame::rename_swamp("dunes");
    let unmet = biome_name(&table, ColorGame::SWAMP_ID, 16).0;
    let handle = chunk_handle(&harness);
    read_columns(&table, handle, &request(30, 0, &[(0, 9)]), &mut samples);

    assert_eq!(unmet, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(biome_name(&table, ColorGame::SWAMP_ID, 16), (AmmlStatus::OK, "dunes".to_string(), 5));
}

// Installs the adapters over the chunk stand-ins and the surface stand-ins of `shape`, minus `without`, and joins.
fn start_with_surfaces(shape: SurfaceShape, without: &[&str]) -> Harness {
    let mut entries = ChunkGame::entries(without, 0x1C8, 1);
    entries.extend(SurfaceGame::entries(shape, without));
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    join(&harness);
    harness
}

// `scan_surfaces` of the column (`x`, `z`) for the block ys `range` with room for `len` samples: the status and the
// samples written.
fn scan(
    table: &AmmlWorldTable,
    source: usize,
    (x, z): (i32, i32),
    (y_min, y_max): (i32, i32),
    len: usize,
) -> (AmmlStatus, Vec<AmmlSurfaceSample>) {
    let mut out = vec![AmmlSurfaceSample::default(); len];
    let mut count = u32::MAX;
    // SAFETY: the table is live; `out` holds `len` samples and `count` is writable.
    let status = unsafe {
        (table.scan_surfaces.unwrap())(
            std::ptr::with_exposed_provenance(source),
            x,
            z,
            y_min,
            y_max,
            out.as_mut_ptr(),
            len as u32,
            &mut count,
        )
    };
    out.truncate(if status == AmmlStatus::OK { count as usize } else { 0 });
    (status, out)
}

// A column (5, 5) with surfaces at 10 (air above), 20 (air above) and 30 (a flower above), and none at 40 (a slab), 50
// (a slab above) or under any block of the fill.
fn surface_column() {
    ChunkGame::fill(5, 5, -64, 10, ChunkKind::Stone);
    ChunkGame::set_block(5, 5, 20, ChunkKind::Stone);
    ChunkGame::set_block(5, 5, 30, ChunkKind::Stone);
    ChunkGame::set_block(5, 5, 31, ChunkKind::Flower);
    ChunkGame::set_block(5, 5, 40, ChunkKind::Slab);
    ChunkGame::set_block(5, 5, 50, ChunkKind::Dirt);
    ChunkGame::set_block(5, 5, 51, ChunkKind::Slab);
}

fn surface_ys(samples: &[AmmlSurfaceSample]) -> Vec<i32> {
    samples.iter().map(|sample| sample.pos.y).collect()
}

#[test]
fn a_scan_gives_the_surfaces_of_a_column_lowest_first_with_the_light_above() {
    let harness = start_with_surfaces(SurfaceShape::Support, &[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    surface_column();

    let (status, samples) = scan(&table, source, (5, 5), (-64, 63), 16);

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(surface_ys(&samples), [10, 20, 30]);
    assert!(samples.iter().all(|sample| (sample.pos.x, sample.pos.z) == (5, 5)));
    // The light is that of the block above: sky `y & 15`, block `15 - (y & 15)` at y + 1.
    let light = |sample: &AmmlSurfaceSample| (sample.sky_light, sample.block_light);
    assert_eq!(samples.iter().map(light).collect::<Vec<_>>(), [(11, 4), (5, 10), (15, 0)]);
    // The brightness is read through the block source of the player.
    assert!(SurfaceGame::light_calls().iter().all(|(at, _)| *at == ActorGame::block_source()));
}

#[test]
fn the_collision_shape_is_asked_of_the_type_with_the_block_the_source_and_no_entity() {
    let harness = start_with_surfaces(SurfaceShape::Support, &[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    surface_column();

    scan(&table, source, (5, 5), (-64, 63), 16);

    let flower = (
        ChunkGame::block_type(ChunkKind::Flower),
        ChunkGame::block(ChunkKind::Flower),
        ActorGame::block_source(),
        [5, 31, 5],
        0,
    );
    assert!(SurfaceGame::collision_calls().contains(&flower), "{:?}", SurfaceGame::collision_calls());
}

// The stand-in scan of one stone block under air, with the support call of `shape` only; the support calls seen.
fn support_calls_of(shape: SurfaceShape) -> Vec<(&'static str, usize, u8, Option<u8>)> {
    let harness = start_with_surfaces(shape, &[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    ChunkGame::fill(5, 5, -64, 10, ChunkKind::Stone);

    let (status, samples) = scan(&table, source, (5, 5), (10, 10), 4);

    assert_eq!((status, surface_ys(&samples)), (AmmlStatus::OK, vec![10]));
    SurfaceGame::support_calls()
}

#[test]
fn support_shape_follows_the_resolved_entry() {
    let rest = ["BlockType.getCollisionShape", "BlockSource.getBrightnessPair"];
    let pick = |support: &str| {
        let chunk = [
            "BlockSource.getChunk",
            "LevelChunk.mSubChunks",
            "LevelChunk.getBlock",
            "SubChunk.size",
            "SubChunk.isPlaceHolderSubChunk",
            "BlockType.mMaterialInline",
            "BlockType.mNameInfo.mFullName",
            "HashedString.layout",
            "Block.mBlockType",
            "Actor.getDimension",
            "Dimension.getBlockSourceFromMainChunkSource",
        ];
        SurfaceShape::pick(&|name| name == support || rest.contains(&name) || chunk.contains(&name))
    };

    assert_eq!(pick("Block.canProvideFullSupport"), Some(SurfaceShape::Support));
    assert_eq!(pick("Block.canProvideFullSupportWithFlag"), Some(SurfaceShape::SupportFlag));
    assert_eq!(pick("Block.isAir"), None);
}

#[test]
fn the_26_63_support_call_takes_the_face_only() {
    // The top face (1), asked of the block at the surface.
    assert_eq!(
        support_calls_of(SurfaceShape::Support),
        [("Block.canProvideFullSupport", ChunkGame::block(ChunkKind::Stone), 1, None)]
    );
}

#[test]
fn the_70_x_support_call_takes_the_face_and_the_flag_zero() {
    assert_eq!(
        support_calls_of(SurfaceShape::SupportFlag),
        [("Block.canProvideFullSupportWithFlag", ChunkGame::block(ChunkKind::Stone), 1, Some(0))]
    );
}

#[test]
fn a_scan_stops_at_the_room_given() {
    let harness = start_with_surfaces(SurfaceShape::Support, &[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    surface_column();

    let (status, samples) = scan(&table, source, (5, 5), (-64, 63), 2);

    assert_eq!((status, surface_ys(&samples)), (AmmlStatus::OK, vec![10, 20]));
    assert_eq!(scan(&table, source, (5, 5), (-64, 63), 0), (AmmlStatus::OK, Vec::new()));
}

#[test]
fn a_scan_is_clamped_to_the_chunk_and_reads_the_top_block_for_the_surface_below_it() {
    let harness = start_with_surfaces(SurfaceShape::Support, &[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    let top = ChunkGame::MAX_Y - 1;
    ChunkGame::set_block(5, 5, top - 1, ChunkKind::Stone);

    let (status, samples) = scan(&table, source, (5, 5), (-5000, 5000), 8);

    // Air at the top block over a block at the top minus one: that surface is the highest one a scan can give.
    assert_eq!((status, surface_ys(&samples)), (AmmlStatus::OK, vec![top - 1]));
    let asked = ChunkGame::asked();
    assert!(asked.contains(&(5, 5, 0)) && asked.contains(&(5, 5, 127)), "{asked:?}");
    assert!(asked.iter().all(|(_, _, y)| (0..128).contains(y)), "{asked:?}");
}

#[test]
fn a_scan_of_a_chunk_that_is_not_loaded_or_not_there_fails() {
    let harness = start_with_surfaces(SurfaceShape::Support, &[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    surface_column();

    // The chunk (1, 0) does not exist in the stand-in.
    assert_eq!(scan(&table, source, (21, 5), (-64, 63), 8).0, AmmlStatus::FAILED);
    ChunkGame::set_load_state(0);
    assert_eq!(scan(&table, source, (5, 5), (-64, 63), 8).0, AmmlStatus::FAILED);
}

#[test]
fn a_scan_that_meets_a_placeholder_block_fails() {
    let harness = start_with_surfaces(SurfaceShape::Support, &[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    ChunkGame::fill(5, 5, -64, 10, ChunkKind::Stone);
    ChunkGame::set_block(5, 5, 11, ChunkKind::Placeholder);

    assert_eq!(scan(&table, source, (5, 5), (-64, 63), 8).0, AmmlStatus::FAILED);
}

#[test]
fn a_scan_off_the_game_thread_or_with_another_block_source_is_refused() {
    let harness = start_with_surfaces(SurfaceShape::Support, &[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    let other = HandleStamp::seal(ActorGame::other_block_source(), 1).unwrap();

    let off_thread =
        std::thread::scope(|scope| scope.spawn(|| scan(&table, source, (5, 5), (-64, 63), 8).0).join().unwrap());

    assert_eq!(off_thread, AmmlStatus::REFUSED);
    assert_eq!(scan(&table, other, (5, 5), (-64, 63), 8).0, AmmlStatus::REFUSED);
    assert_eq!(scan(&table, 0, (5, 5), (-64, 63), 8).0, AmmlStatus::REFUSED);
}

#[test]
fn null_pointers_of_the_scan_are_invalid_arguments() {
    let harness = start_with_surfaces(SurfaceShape::Support, &[]);
    let table = table(&harness);
    let source = std::ptr::with_exposed_provenance(block_source_handle(&harness));
    let mut out = [AmmlSurfaceSample::default(); 2];
    let mut count = 0u32;

    // SAFETY: the table is live; the loader checks the null pointers before it writes.
    let (no_count, no_out) = unsafe {
        (
            (table.scan_surfaces.unwrap())(source, 5, 5, 0, 10, out.as_mut_ptr(), 2, std::ptr::null_mut()),
            (table.scan_surfaces.unwrap())(source, 5, 5, 0, 10, std::ptr::null_mut(), 2, &mut count),
        )
    };

    assert_eq!((no_count, no_out), (AmmlStatus::INVALID_ARGUMENT, AmmlStatus::INVALID_ARGUMENT));
}

// Whether `world.scan_surfaces` is offered without the entry `missing`; the chunk reads are offered in each case.
fn scan_area_without(missing: &str) -> bool {
    let harness = start_with_surfaces(SurfaceShape::Support, &[missing]);
    assert!(harness.state.areas().is_available(WorldAdapter::CHUNK_READ, 1), "{missing}");
    harness.state.areas().is_available(WorldAdapter::SCAN_SURFACES, 1)
}

#[test]
fn the_scan_area_is_not_offered_without_the_support_entry() {
    assert!(!scan_area_without("Block.canProvideFullSupport"));
}

#[test]
fn the_scan_area_is_not_offered_without_the_collision_shape_entry() {
    assert!(!scan_area_without("BlockType.getCollisionShape"));
}

#[test]
fn the_scan_area_is_not_offered_without_the_brightness_entry() {
    assert!(!scan_area_without("BlockSource.getBrightnessPair"));
}

#[test]
fn the_scan_area_is_offered_on_the_70_x_support_shape() {
    let harness = start_with_surfaces(SurfaceShape::SupportFlag, &[]);

    assert!(harness.state.areas().is_available(WorldAdapter::SCAN_SURFACES, 1));
}

// The module of the mod that `start_shapes` loads.
const SHAPES_MODULE: &str = "@events/events";

// A shape op of `kind` with id `id`, from (1, 2, 3) to (4, 5, 6), in opaque red.
fn shape_op(kind: u8, id: u64) -> AmmlShapeOp {
    AmmlShapeOp {
        kind,
        _reserved: [0; 7],
        id,
        from: AmmlVec3 { x: 1.0, y: 2.0, z: 3.0 },
        to: AmmlVec3 { x: 4.0, y: 5.0, z: 6.0 },
        color: AmmlRgba { r: 255, g: 0, b: 51, a: 255 },
        _pad: 0,
    }
}

fn shapes_apply(table: &AmmlWorldTable, dimension: i32, ops: &[AmmlShapeOp]) -> (AmmlStatus, AmmlApplyStats) {
    let mut stats = AmmlApplyStats { delivered: 99, elapsed_us: 99 };
    let count = u32::try_from(ops.len()).unwrap();
    // SAFETY: the table is live; the module, the ops and the stats live for the call.
    let status = unsafe {
        (table.shapes_apply.unwrap())(AmmlStr::from_str(SHAPES_MODULE), dimension, ops.as_ptr(), count, &mut stats)
    };
    (status, stats)
}

fn target_epoch(table: &AmmlWorldTable) -> (AmmlStatus, u32) {
    let mut out = 0u32;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.shapes_target_epoch.unwrap())(&mut out) };
    (status, out)
}

// The session, client, free camera and shapes adapters over the free camera and shape stand-ins, one mod enabled
// (declaring `world.shapes` when `declares`) and a joined world.
fn start_shapes(declares: bool) -> Harness {
    FreeCameraGame::reset(&["minecraft:free"]);
    let harness = Harness::start_with_entries(&FreeCameraGame::shape_entries());
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    CameraAdapter::install(harness.state);
    FreeCameraAdapter::install(harness.state);
    ShapesAdapter::install(harness.state);
    let dll =
        if declares { EventsMod::build("world-shapes", &["world-shapes"]) } else { EventsMod::build("plain", &[]) };
    harness.install_mod("events", "events", &dll, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    assert!(loader.enable_all().iter().all(|report| report.is_ok()));
    SessionGame::frame();
    SessionGame::set_player(FreeCameraGame::player());
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    SessionGame::frame();
    harness
}

// The shape lines of the stand-in handlers so far: each `shapes` call line and its payload lines.
fn shape_log() -> Vec<String> {
    FreeCameraGame::log().into_iter().filter(|line| line.starts_with("shape")).collect()
}

#[test]
fn apply_before_capture_is_not_ready() {
    let harness = start_shapes(true);
    let table = table(&harness);
    assert!(harness.state.areas().is_available(ShapesAdapter::AREA, 1));

    let (status, _) = shapes_apply(&table, 0, &[shape_op(AmmlShapeOp::LINE, 1)]);

    assert_eq!(status, AmmlStatus::NOT_READY);
    assert_eq!(target_epoch(&table).0, AmmlStatus::NOT_READY);
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.shapes_max_batch.unwrap())() }, 800);
    assert!(FreeCameraGame::created().is_empty());
    assert!(shape_log().is_empty());
}

#[test]
fn batch_over_max_is_refused() {
    let harness = start_shapes(true);
    let table = table(&harness);
    FreeCameraGame::chunk_publisher();
    let ops: Vec<AmmlShapeOp> = (0..801).map(|id| shape_op(AmmlShapeOp::LINE, id)).collect();

    assert_eq!(shapes_apply(&table, 0, &ops).0, AmmlStatus::REFUSED);

    assert!(FreeCameraGame::created().is_empty());
    // A batch of the maximum goes out in one packet.
    let (status, stats) = shapes_apply(&table, 0, &ops[..800]);
    assert_eq!((status, stats.delivered), (AmmlStatus::OK, 800));
    assert_eq!(shape_log().len(), 801);
}

#[test]
fn apply_after_capture_hands_the_shapes_to_the_captured_handler() {
    let harness = start_shapes(true);
    let table = table(&harness);
    FreeCameraGame::chunk_publisher();

    let ops = [shape_op(AmmlShapeOp::LINE, 1), shape_op(AmmlShapeOp::BOX, 2), shape_op(AmmlShapeOp::ARROW, 3)];
    let (status, stats) = shapes_apply(&table, 1, &ops);
    let (removed, removed_stats) = shapes_apply(&table, 1, &[shape_op(AmmlShapeOp::REMOVE, 2)]);

    assert_eq!((status, stats.delivered), (AmmlStatus::OK, 3));
    assert_eq!((removed, removed_stats.delivered), (AmmlStatus::OK, 1));
    // One PrimitiveShapesPacket (id 328) serves both deliveries; its vector is empty after each.
    assert_eq!(FreeCameraGame::created(), [FreeCameraGame::SHAPES_ID]);
    let packet = FreeCameraGame::packets()[0];
    let [first, last, end] = FreeCameraGame::shape_vector(packet);
    assert_eq!(first, last);
    assert!(first != 0 && end - first >= 3 * FreeCameraGame::PAYLOAD);
    let log = shape_log();
    let handler = FreeCameraGame::handler();
    assert_eq!(log[0], format!("shapes handler={handler:#x} id=invalid count=3"));
    let common = "rot=0,0,0/1 scale=1/1 color=1.000,0.000,0.200,1.000/1 dim=1/1 time=0 attached=0 distance=0";
    let line = &log[1];
    assert!(line.contains("type=0/1 loc=1,2,3/1") && line.contains(common), "{line}");
    assert!(line.ends_with("extra=4 end=4,5,6"), "{line}");
    let shape_box = &log[2];
    assert!(shape_box.contains("type=1/1 loc=1,2,3/1") && shape_box.ends_with("extra=3 end=4,5,6"), "{shape_box}");
    let arrow = &log[3];
    assert!(arrow.contains("type=5/1") && arrow.contains(common), "{arrow}");
    assert!(arrow.ends_with("extra=1 end=4,5,6/1 head=0.4/1,0.2/1 segments=8/1"), "{arrow}");
    // Three game network ids, one per shape; the remove carries the box's id and nothing else.
    let net = |line: &str| line.split(' ').nth(1).unwrap().to_string();
    assert!(net(&log[1]) != net(&log[2]) && net(&log[2]) != net(&log[3]));
    assert_eq!(log[4], format!("shapes handler={handler:#x} id=invalid count=1"));
    assert!(log[5].starts_with(&format!("shape {} type=0/0 loc=0,0,0/0", net(&log[2]))), "{}", log[5]);
    assert!(log[5].ends_with("extra=0"), "{}", log[5]);
}

#[test]
fn target_epoch_is_the_handler_generation_and_changes_with_a_new_world() {
    let harness = start_shapes(true);
    let table = table(&harness);
    FreeCameraGame::chunk_publisher();
    let (status, first) = target_epoch(&table);
    assert_eq!(status, AmmlStatus::OK);
    assert_ne!(first, 0);

    SessionGame::exit();
    assert_eq!(target_epoch(&table).0, AmmlStatus::NOT_READY);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    FreeCameraGame::chunk_publisher();

    let (status, second) = target_epoch(&table);
    assert_eq!(status, AmmlStatus::OK);
    assert_ne!(second, first);
}

#[test]
fn disable_removes_the_live_shapes_of_the_mod() {
    let harness = start_shapes(true);
    let table = table(&harness);
    FreeCameraGame::chunk_publisher();
    let ops = [shape_op(AmmlShapeOp::LINE, 1), shape_op(AmmlShapeOp::ARROW, 2), shape_op(AmmlShapeOp::LINE, 3)];
    assert_eq!(shapes_apply(&table, 0, &ops).0, AmmlStatus::OK);
    assert_eq!(shapes_apply(&table, 0, &[shape_op(AmmlShapeOp::REMOVE, 3)]).0, AmmlStatus::OK);
    let drawn = shape_log();

    ShapesAdapter::get().unwrap().disable_owner(0);

    let log = shape_log();
    let removes = &log[drawn.len()..];
    assert_eq!(removes.len(), 3, "{removes:?}");
    assert!(removes[0].ends_with("count=2"), "{removes:?}");
    let net = |line: &str| line.split(' ').nth(1).unwrap().to_string();
    assert_eq!([net(&removes[1]), net(&removes[2])], [net(&drawn[1]), net(&drawn[2])]);
    assert!(removes[1..].iter().all(|line| line.contains("type=0/0")), "{removes:?}");
    // The packet stays for the world; it goes back at the world leave.
    assert_eq!(FreeCameraGame::uses(), FreeCameraGame::USES + 1);
}

#[test]
fn world_leave_gives_the_packet_back_and_sends_nothing() {
    let harness = start_shapes(true);
    let table = table(&harness);
    FreeCameraGame::chunk_publisher();
    assert_eq!(shapes_apply(&table, 0, &[shape_op(AmmlShapeOp::BOX, 1)]).0, AmmlStatus::OK);
    let delivered = shape_log().len();

    SessionGame::exit();
    ShapesAdapter::get().unwrap().disable_owner(0);

    assert_eq!(shape_log().len(), delivered);
    assert_eq!(FreeCameraGame::uses(), FreeCameraGame::USES);
    assert_eq!(shapes_apply(&table, 0, &[shape_op(AmmlShapeOp::BOX, 1)]).0, AmmlStatus::NOT_READY);
}

#[test]
fn a_shutdown_disable_sends_nothing_and_runs_no_game_code_for_the_packet() {
    let harness = start_shapes(true);
    let table = table(&harness);
    FreeCameraGame::chunk_publisher();
    assert_eq!(shapes_apply(&table, 0, &[shape_op(AmmlShapeOp::LINE, 1)]).0, AmmlStatus::OK);
    let delivered = shape_log().len();

    ModLoader::new(harness.state).shutdown();

    assert_eq!(shape_log().len(), delivered);
    assert_eq!(FreeCameraGame::uses(), FreeCameraGame::USES + 1);
}

#[test]
fn apply_off_the_game_thread_is_refused() {
    let harness = start_shapes(true);
    let table = table(&harness);
    FreeCameraGame::chunk_publisher();

    let status =
        std::thread::spawn(move || shapes_apply(&table, 0, &[shape_op(AmmlShapeOp::LINE, 1)]).0).join().unwrap();

    assert_eq!(status, AmmlStatus::REFUSED);
    assert!(FreeCameraGame::created().is_empty());
}

#[test]
fn a_bad_op_is_an_invalid_argument_and_nothing_goes_out() {
    let harness = start_shapes(true);
    let table = table(&harness);
    FreeCameraGame::chunk_publisher();
    let unknown = shape_op(9, 1);
    let infinite = AmmlShapeOp { to: AmmlVec3 { x: f32::INFINITY, y: 0.0, z: 0.0 }, ..shape_op(AmmlShapeOp::LINE, 2) };

    assert_eq!(shapes_apply(&table, 0, &[shape_op(AmmlShapeOp::LINE, 1), unknown]).0, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(shapes_apply(&table, 0, &[infinite]).0, AmmlStatus::INVALID_ARGUMENT);
    // SAFETY: the table is live; null ops with a count are refused before they are read.
    let null = unsafe {
        (table.shapes_apply.unwrap())(AmmlStr::from_str(SHAPES_MODULE), 0, std::ptr::null(), 1, std::ptr::null_mut())
    };
    assert_eq!(null, AmmlStatus::INVALID_ARGUMENT);

    assert!(shape_log().is_empty());
    // An empty batch goes out as nothing.
    let (status, stats) = shapes_apply(&table, 0, &[]);
    assert_eq!((status, stats.delivered), (AmmlStatus::OK, 0));
    assert!(FreeCameraGame::created().is_empty());
}

#[test]
fn without_a_mod_that_declares_the_area_the_handler_is_not_captured() {
    let harness = start_shapes(false);
    let table = table(&harness);

    FreeCameraGame::chunk_publisher();

    assert_eq!(shapes_apply(&table, 0, &[shape_op(AmmlShapeOp::LINE, 1)]).0, AmmlStatus::NOT_READY);
}

#[test]
fn the_shapes_area_is_not_offered_with_a_payload_size_that_does_not_fit() {
    FreeCameraGame::reset(&["minecraft:free"]);
    let mut entries = FreeCameraGame::shape_entries();
    for (name, resolved) in &mut entries {
        if *name == "PrimitiveShapeDataPayload.size" {
            resolved.value = 0xC0;
        }
    }
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    FreeCameraAdapter::install(harness.state);

    ShapesAdapter::install(harness.state);

    assert!(!harness.state.areas().is_available(ShapesAdapter::AREA, 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "shape layout mismatch");
    assert!(log.contains("PrimitiveShapeDataPayload.size"), "{log}");
}

#[test]
fn the_shape_block_grows_frees_the_old_block_and_goes_with_the_packet() {
    let harness = start_shapes(true);
    let table = table(&harness);
    FreeCameraGame::destroy_shapes_packets();
    FreeCameraGame::chunk_publisher();
    let payload = FreeCameraGame::PAYLOAD;

    assert_eq!(shapes_apply(&table, 0, &[shape_op(AmmlShapeOp::LINE, 1)]).0, AmmlStatus::OK);
    let packet = FreeCameraGame::packets()[0];
    let [small, _, small_end] = FreeCameraGame::shape_vector(packet);
    assert_eq!(small_end - small, payload);
    assert!(HeapProbe::allocated().contains(&(small, payload)));

    // 30 payloads are 1590h bytes, over the 4096-byte big-allocation threshold.
    let ops: Vec<AmmlShapeOp> = (0..30).map(|id| shape_op(AmmlShapeOp::LINE, id)).collect();
    let (status, stats) = shapes_apply(&table, 0, &ops);
    assert_eq!((status, stats.delivered), (AmmlStatus::OK, 30));
    let [big, last, big_end] = FreeCameraGame::shape_vector(packet);
    assert_eq!(big_end - big, 30 * payload);
    assert_eq!(last, big);
    // The first block went back with its own size; the new one is a big allocation: the probe gave the raw block of
    // 1590h + 27h bytes, and the loader keeps the raw pointer just before the aligned block.
    assert!(HeapProbe::freed().contains(&(small, payload)), "{:?}", HeapProbe::freed());
    // SAFETY: a big allocation of the loader's `MsvcHeap` keeps the raw pointer in the 8 bytes before the block.
    let raw = unsafe { std::ptr::with_exposed_provenance::<usize>(big - 8).read_unaligned() };
    assert!(HeapProbe::allocated().contains(&(raw, 30 * payload + 0x27)), "{:?}", HeapProbe::allocated());
    assert!(!HeapProbe::freed().iter().any(|(address, _)| *address == raw));

    SessionGame::exit();

    // The release of the packet's one use destroys it, and its empty vector frees the big block with the size the loader
    // allocated.
    assert_eq!(FreeCameraGame::destroyed(), [packet]);
    let freed: Vec<(usize, usize)> = HeapProbe::freed().into_iter().filter(|(address, _)| *address == raw).collect();
    assert_eq!(freed, [(raw, 30 * payload + 0x27)]);
}

// Installs the adapters over the chunk stand-ins and the ray stand-ins, minus `without`, and joins.
fn start_with_rays(without: &[&str]) -> Harness {
    start_with_rays_in_order(without, 0)
}

// `start_with_rays` with `BlockSource.fetchEntitiesSpanOrder` at `order`.
fn start_with_rays_in_order(without: &[&str], order: u64) -> Harness {
    let mut entries = ChunkGame::entries(without, 0x1C8, 1);
    entries.extend(RayGame::entries_in_order(without, order));
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    join(&harness);
    harness
}

fn vec3(x: f32, y: f32, z: f32) -> AmmlVec3 {
    AmmlVec3 { x, y, z }
}

const EVERY_BLOCK: AmmlRayOptions = AmmlRayOptions { every_block: 1, outline: 1, _reserved: [0; 2], max_steps: 0 };

// `raycast` of `source` from `from` to `to`; the hit keeps its marker bytes while `out` was not written.
fn raycast(
    table: &AmmlWorldTable,
    source: usize,
    from: AmmlVec3,
    to: AmmlVec3,
    options: AmmlRayOptions,
) -> (AmmlStatus, AmmlBlockHit) {
    let mut out = AmmlBlockHit {
        hit: 0xEE,
        face: AmmlFace(0xEE),
        _reserved: [0xEE; 2],
        pos: AmmlBlockPos { x: -1, y: -1, z: -1 },
        point: vec3(-1.0, -1.0, -1.0),
    };
    // SAFETY: the table is live and `out` is writable.
    let status =
        unsafe { (table.raycast.unwrap())(std::ptr::with_exposed_provenance(source), from, to, options, &mut out) };
    (status, out)
}

// `actors_in` of `source` for `aabb` with room for `cap` boxes: the status and the boxes written.
fn actors_in(table: &AmmlWorldTable, source: usize, aabb: AmmlAabb, cap: u32) -> (AmmlStatus, Vec<AmmlActorBox>) {
    let mut out = vec![AmmlActorBox::default(); cap as usize];
    let mut count = u32::MAX;
    // SAFETY: the table is live; `out` holds `cap` boxes and `count` is writable.
    let status = unsafe {
        (table.actors_in.unwrap())(std::ptr::with_exposed_provenance(source), aabb, out.as_mut_ptr(), cap, &mut count)
    };
    out.truncate(if status == AmmlStatus::OK { count as usize } else { 0 });
    (status, out)
}

fn stone_at(pos: [i32; 3], face: u8) -> Candidate {
    Candidate {
        block: ChunkGame::block(ChunkKind::Stone),
        pos,
        face,
        point: [pos[0] as f32 + 0.5, pos[1] as f32 + 1.0, pos[2] as f32 + 0.5],
    }
}

fn flower_at(pos: [i32; 3]) -> Candidate {
    Candidate { block: ChunkGame::block(ChunkKind::Flower), pos, face: 1, point: [0.5, 0.5, 0.5] }
}

const QUERY: AmmlAabb =
    AmmlAabb { min: AmmlVec3 { x: -8.0, y: 56.0, z: -8.0 }, max: AmmlVec3 { x: 8.0, y: 72.0, z: 8.0 } };

#[test]
fn non_finite_ray_is_invalid_argument() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);

    let nan_from = raycast(&table, source, vec3(f32::NAN, 64.0, 0.0), vec3(0.0, 60.0, 0.0), EVERY_BLOCK);
    let infinite_to = raycast(&table, source, vec3(0.0, 64.0, 0.0), vec3(0.0, f32::INFINITY, 0.0), EVERY_BLOCK);

    assert_eq!(nan_from.0, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(infinite_to.0, AmmlStatus::INVALID_ARGUMENT);
    assert!(RayGame::clip_calls().is_empty());
    // SAFETY: the table is live; a null out is refused before it is written.
    let null = unsafe {
        (table.raycast.unwrap())(
            std::ptr::with_exposed_provenance(source),
            vec3(0.0, 64.0, 0.0),
            vec3(0.0, 60.0, 0.0),
            EVERY_BLOCK,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(null, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn a_ray_gives_the_block_face_and_point_of_the_tile_hit() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    RayGame::set_candidates(&[stone_at([3, 60, -2], 1)]);

    let (status, hit) = raycast(&table, source, vec3(3.5, 64.0, -1.5), vec3(3.5, 50.0, -1.5), EVERY_BLOCK);

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!((hit.hit, hit.face), (1, AmmlFace::UP));
    assert_eq!(hit.pos, AmmlBlockPos { x: 3, y: 60, z: -2 });
    assert_eq!(hit.point, vec3(3.5, 61.0, -1.5));
}

#[test]
fn a_ray_that_hits_nothing_is_ok_with_no_hit() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);

    let (status, hit) = raycast(&table, source, vec3(0.0, 64.0, 0.0), vec3(0.0, 50.0, 0.0), EVERY_BLOCK);

    assert_eq!((status, hit.hit), (AmmlStatus::OK, 0));
    assert_eq!(hit.pos, AmmlBlockPos::default());
}

#[test]
fn the_clip_gets_the_points_the_shape_type_the_steps_and_no_actor() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    let collision = AmmlRayOptions { every_block: 1, outline: 0, _reserved: [0; 2], max_steps: 12 };

    raycast(&table, source, vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), EVERY_BLOCK);
    raycast(&table, source, vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), collision);

    let calls = RayGame::clip_calls();
    assert_eq!(calls.len(), 2);
    let first = calls[0];
    assert_eq!(first.source, ActorGame::block_source());
    assert_eq!((first.from, first.to), ([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]));
    // The outline shape type 0, the game's own 200 steps for 0, no liquid, no border or full-only rule, no actor, a
    // callable `std::function`, no stop on a liquid.
    assert_eq!((first.shape, first.steps), (0, 200));
    assert_eq!((first.liquid, first.border, first.full_only, first.actor, first.stop_on_liquid), (0, 0, 0, 0, 0));
    assert!(first.has_callable);
    // The collision shape type 1 and the steps asked.
    assert_eq!((calls[1].shape, calls[1].steps), (1, 12));
}

#[test]
fn steps_above_the_largest_int_are_clamped() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    let many = AmmlRayOptions { max_steps: u32::MAX, ..EVERY_BLOCK };

    raycast(&table, source, vec3(0.0, 64.0, 0.0), vec3(0.0, 50.0, 0.0), many);

    assert_eq!(RayGame::clip_calls()[0].steps, i32::MAX);
}

#[test]
fn every_block_accepts_a_flower_and_solid_only_passes_through_it() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    RayGame::set_candidates(&[flower_at([0, 62, 0]), stone_at([0, 60, 0], 1)]);
    let solid_only = AmmlRayOptions { every_block: 0, ..EVERY_BLOCK };

    let every = raycast(&table, source, vec3(0.5, 64.0, 0.5), vec3(0.5, 50.0, 0.5), EVERY_BLOCK);
    let solid = raycast(&table, source, vec3(0.5, 64.0, 0.5), vec3(0.5, 50.0, 0.5), solid_only);

    assert_eq!((every.0, every.1.pos.y), (AmmlStatus::OK, 62));
    assert_eq!((solid.0, solid.1.pos.y), (AmmlStatus::OK, 60));
    // The function is asked with the block source of the player.
    assert!(RayGame::asked_blocks().iter().all(|(at, _)| *at == ActorGame::block_source()));
}

#[test]
fn the_weak_entity_reference_of_the_result_is_given_back() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    RayGame::set_weak(true);

    raycast(&table, source, vec3(0.0, 64.0, 0.0), vec3(0.0, 50.0, 0.0), EVERY_BLOCK);
    assert_eq!(RayGame::weak_state(), (1, 0));
    RayGame::set_candidates(&[stone_at([0, 60, 0], 1)]);
    raycast(&table, source, vec3(0.0, 64.0, 0.0), vec3(0.0, 50.0, 0.0), EVERY_BLOCK);

    // The second release takes the last weak count, so the block deletes itself once.
    assert_eq!(RayGame::weak_state(), (0, 1));
}

#[test]
fn a_ray_with_a_block_source_of_another_world_or_off_the_game_thread_is_refused() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    let down = (vec3(0.0, 64.0, 0.0), vec3(0.0, 50.0, 0.0));

    let other = raycast(&table, ActorGame::other_block_source(), down.0, down.1, EVERY_BLOCK);
    let off_thread = std::thread::spawn(move || raycast(&table, source, down.0, down.1, EVERY_BLOCK).0).join().unwrap();

    assert_eq!(other.0, AmmlStatus::REFUSED);
    assert_eq!(off_thread, AmmlStatus::REFUSED);
    assert!(RayGame::clip_calls().is_empty());
}

#[test]
fn the_ray_areas_are_not_offered_without_their_entries() {
    let harness = start_with_rays(&["BlockSource.clip", "BlockSource.fetchEntities"]);
    let table = table(&harness);
    let source = block_source_handle(&harness);

    assert!(!harness.state.areas().is_available(WorldAdapter::RAYCAST, 1));
    assert!(!harness.state.areas().is_available(WorldAdapter::ACTORS_IN, 1));
    let down = raycast(&table, source, vec3(0.0, 64.0, 0.0), vec3(0.0, 50.0, 0.0), EVERY_BLOCK);
    assert_eq!(down.0, AmmlStatus::NOT_AVAILABLE);
    assert_eq!(actors_in(&table, source, QUERY, 4).0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn actors_cap_256_is_kept() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    for index in 0..300 {
        let x = index as f32 * 0.01;
        RayGame::add_actor([x, 60.0, 0.0, x + 0.6, 61.8, 0.6]);
    }

    let (status, boxes) = actors_in(&table, source, QUERY, 256);

    assert_eq!((status, boxes.len()), (AmmlStatus::OK, 256));
    assert_eq!(boxes[255].aabb.min.x, 255.0 * 0.01);
}

#[test]
fn actors_in_marks_the_local_player_and_copies_each_box() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    RayGame::add_actor([1.0, 60.0, 1.0, 1.6, 61.8, 1.6]);
    RayGame::add_existing_actor(ActorGame::player());

    let (status, boxes) = actors_in(&table, source, QUERY, 8);

    assert_eq!((status, boxes.len()), (AmmlStatus::OK, 2));
    assert_eq!(boxes[0].aabb, AmmlAabb { min: vec3(1.0, 60.0, 1.0), max: vec3(1.6, 61.8, 1.6) });
    assert_eq!(boxes[0].is_local_player, 0);
    assert_eq!(boxes[1].aabb, AmmlAabb { min: vec3(1.2, 64.0, -3.3), max: vec3(1.8, 65.8, -2.7) });
    assert_eq!(boxes[1].is_local_player, 1);
    // The fetch got the block source of the player, the box asked, no actor left out and the flags 1 and 0.
    let (at, except, aabb, first, second) = RayGame::fetch_calls()[0];
    assert_eq!((at, except, first, second), (ActorGame::block_source(), 0, 1, 0));
    assert_eq!(aabb, [-8.0, 56.0, -8.0, 8.0, 72.0, 8.0]);
}

// The boxes `actors_in` gives for two stand-in actors when the derived order is `order` and the game writes the span
// in `written`.
fn two_actors_in(order: u64, written: SpanOrder) -> (AmmlStatus, Vec<AmmlActorBox>) {
    let harness = start_with_rays_in_order(&[], order);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    RayGame::add_actor([1.0, 60.0, 1.0, 1.6, 61.8, 1.6]);
    RayGame::add_actor([2.0, 60.0, 2.0, 2.6, 61.8, 2.6]);
    RayGame::set_order(written);
    actors_in(&table, source, QUERY, 8)
}

#[test]
fn the_derived_order_0_reads_the_count_first_span_of_26_63() {
    let (status, boxes) = two_actors_in(0, SpanOrder::CountFirst);

    assert_eq!((status, boxes.len()), (AmmlStatus::OK, 2));
    assert_eq!(boxes[1].aabb.min, vec3(2.0, 60.0, 2.0));
}

#[test]
fn the_derived_order_1_reads_the_pointer_first_span_of_70_26() {
    let (status, boxes) = two_actors_in(1, SpanOrder::PointerFirst);

    assert_eq!((status, boxes.len()), (AmmlStatus::OK, 2));
    assert_eq!(boxes[1].aabb.min, vec3(2.0, 60.0, 2.0));
}

#[test]
fn a_span_in_the_other_order_than_the_derived_one_fails() {
    let (status, boxes) = two_actors_in(1, SpanOrder::CountFirst);

    assert_eq!((status, boxes.len()), (AmmlStatus::FAILED, 0));
}

#[test]
fn a_span_order_that_is_neither_0_nor_1_offers_no_actors_area() {
    let harness = start_with_rays_in_order(&[], 2);

    assert!(!harness.state.areas().is_available(WorldAdapter::ACTORS_IN, 1));
    assert!(harness.state.areas().is_available(WorldAdapter::RAYCAST, 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "actor span mismatch");
    assert!(log.contains("BlockSource.fetchEntitiesSpanOrder"), "{log}");
}

#[test]
fn without_the_span_order_entry_the_actors_area_is_not_offered() {
    let harness = start_with_rays(&["BlockSource.fetchEntitiesSpanOrder"]);

    assert!(!harness.state.areas().is_available(WorldAdapter::ACTORS_IN, 1));
    assert!(harness.state.areas().is_available(WorldAdapter::RAYCAST, 1));
}

#[test]
fn an_empty_span_gives_no_box_and_a_span_that_does_not_fit_the_order_fails() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);

    let empty = actors_in(&table, source, QUERY, 8);
    RayGame::set_raw_span([0x7FF0_0000_0000, 0x7FF0_0000_1000]);
    let two_pointers = actors_in(&table, source, QUERY, 8);
    RayGame::set_raw_span([3, 0]);
    let count_without_data = actors_in(&table, source, QUERY, 8);

    assert_eq!((empty.0, empty.1.len()), (AmmlStatus::OK, 0));
    assert_eq!(two_pointers.0, AmmlStatus::FAILED);
    assert_eq!(count_without_data.0, AmmlStatus::FAILED);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "actor span mismatch");
    assert_eq!(log.matches("actor span mismatch").count(), 1, "{log}");
}

#[test]
fn actors_in_off_the_game_thread_or_with_a_bad_box_is_refused() {
    let harness = start_with_rays(&[]);
    let table = table(&harness);
    let source = block_source_handle(&harness);
    let bad = AmmlAabb { min: vec3(f32::NAN, 0.0, 0.0), max: vec3(1.0, 1.0, 1.0) };

    let off_thread = std::thread::spawn(move || actors_in(&table, source, QUERY, 4).0).join().unwrap();
    let not_finite = actors_in(&table, source, bad, 4).0;

    assert_eq!(off_thread, AmmlStatus::REFUSED);
    assert_eq!(not_finite, AmmlStatus::INVALID_ARGUMENT);
    assert!(RayGame::fetch_calls().is_empty());
}
