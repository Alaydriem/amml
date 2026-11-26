use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use amml::{
    ActorAdapter, BlockHandles, BlockReadAdapter, ClientAdapter, ModListener, ModLoader, SessionAdapter, WorldAdapter,
    WorldRenderAdapter,
};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlBlockGrid, AmmlBlockHandle, AmmlBlockPos, AmmlEventContext, AmmlEventHeader, AmmlMeshBuilt,
    AmmlMeshGroup, AmmlMesherOptions, AmmlRenderTable, AmmlServices, AmmlStatus, AmmlStr, AmmlTransform, AmmlVec3,
    WorldRendered,
};
use amml_runtime::{ModRegistry, ResourceId};

use crate::support::{
    ChunkGame, ChunkKind, ClientGame, EventsMod, GAME_CLIENT_A, Harness, HeapProbe, LogFile, MeshGame, MesherGame,
    RenderGame, SessionGame,
};

// The module of the mod that `start` loads; its mod index is 0.
const MODULE: &str = "@events/events";
const OWNER: usize = 0;
// sizeof(BlockTessellator) on both gate builds (survey section 7, row M2).
const TESSELLATOR_SIZE: usize = 0x479E0;

// The address of the loader's render table, for the listener.
static TABLE: AtomicUsize = AtomicUsize::new(0);
// The grid the listener builds at each dispatch, and what each build gave.
static PLAN: Mutex<Option<Plan>> = Mutex::new(None);
static BUILDS: Mutex<Vec<Built>> = Mutex::new(Vec::new());
// What the listener runs at each dispatch, with the frame id.
type Action = Box<dyn Fn(u64) + Send>;
static ACTION: Mutex<Option<Action>> = Mutex::new(None);

#[derive(Clone)]
struct Plan {
    dims: [u32; 3],
    origin: AmmlBlockPos,
    cells: Vec<u16>,
    palette: Vec<AmmlBlockHandle>,
    options: AmmlMesherOptions,
}

#[derive(Clone, Debug)]
struct Built {
    status: AmmlStatus,
    out: AmmlMeshBuilt,
    skipped: Vec<u32>,
    // What a draw of the mesh id of an empty build in the same dispatch answered.
    draw: AmmlStatus,
}

fn table() -> AmmlRenderTable {
    // SAFETY: `TABLE` holds the address of the loader's render table, which lives for the process.
    unsafe { AmmlRenderTable::read(std::ptr::with_exposed_provenance(TABLE.load(Ordering::SeqCst))) }.unwrap()
}

fn mesher_build(frame: u64, plan: &Plan) -> Built {
    let grid = AmmlBlockGrid {
        dims: plan.dims,
        origin: plan.origin,
        cells: plan.cells.as_ptr(),
        cell_count: plan.cells.len() as u32,
        palette: plan.palette.as_ptr(),
        palette_len: plan.palette.len() as u32,
    };
    let mut out = AmmlMeshBuilt::default();
    let mut skipped = vec![u32::MAX; plan.cells.len()];
    // SAFETY: the table is live; the module, the grid, its cells and palette, the options and the outputs live for the
    // call.
    let status = unsafe {
        (table().mesher_build.unwrap())(
            AmmlStr::from_str(MODULE),
            frame,
            &raw const grid,
            &raw const plan.options,
            &raw mut out,
            skipped.as_mut_ptr(),
            skipped.len() as u32,
        )
    };
    skipped.truncate(out.skipped_count as usize);
    Built { status, out, skipped, draw: AmmlStatus::OK }
}

fn draw(frame: u64, mesh: amml_abi::AmmlMeshId) -> AmmlStatus {
    let transform = AmmlTransform { origin: ORIGIN, offset: AmmlVec3::default(), scale: 1.0, yaw_degrees: 0.0 };
    // SAFETY: the table is live; the module and the transform live for the call.
    unsafe { (table().mesh_draw.unwrap())(AmmlStr::from_str(MODULE), frame, mesh, &raw const transform) }
}

unsafe extern "C" fn on_rendered(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `render.world_rendered` with a `WorldRendered`.
    let frame = unsafe { &*event.cast::<WorldRendered>() }.frame_id;
    let plan = PLAN.lock().unwrap().clone();
    if let Some(plan) = plan {
        let mut built = mesher_build(frame, &plan);
        // Only the id of an empty build is drawn: a block mesh draw needs the client's item renderer, which the
        // client stand-in of this harness does not have.
        if built.status == AmmlStatus::OK && built.out.quads == 0 {
            built.draw = draw(frame, built.out.mesh);
        }
        BUILDS.lock().unwrap().push(built);
    }
    if let Some(action) = ACTION.lock().unwrap().as_ref() {
        action(frame);
    }
    AmmlStatus::OK
}

fn builds() -> Vec<Built> {
    BUILDS.lock().unwrap().clone()
}

// The adapters of the block reads and the world render over the stand-ins, one loaded mod, the render table found and a
// `WorldRendered` listener that builds the plan.
fn start() -> Harness {
    start_without(&[])
}

// As `start`, with the game entries `without` left out.
fn start_without(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&MesherGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    BlockReadAdapter::install(harness.state);
    WorldRenderAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let render = unsafe { AmmlServices::render(services) }.unwrap();
    TABLE.store(render.expose_provenance(), Ordering::SeqCst);
    let listener = ModListener { owner: OWNER, callback: on_rendered, user: 0 };
    assert!(harness.state.events().listen("render.world_rendered", 1, AmmlBand::NORMAL, listener).is_ok());
    harness
}

// A frame, then a joined world whose local player is the stand-in player, and a camera for the render.
fn join(harness: &Harness) {
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(crate::support::ActorGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    assert_ne!(harness.state.events().gate().epoch().0, 0);
    RenderGame::set_camera(Some([0.0, 70.0, 0.0]));
}

// The block handle of the stand-in block `address` for the loaded mod in the current world.
fn handle(harness: &Harness, address: usize) -> AmmlBlockHandle {
    let epoch = harness.state.events().gate().epoch();
    let id = harness.state.resources().insert(BlockHandles::BLOCK, OWNER, epoch, address).unwrap();
    AmmlBlockHandle(id.0)
}

// The grid index of (x, y, z): x first, then z, then y.
fn cell(dims: [u32; 3], x: u32, y: u32, z: u32) -> usize {
    ((y * dims[2] + z) * dims[0] + x) as usize
}

const DIMS: [u32; 3] = [3, 3, 4];
const ORIGIN: AmmlBlockPos = AmmlBlockPos { x: 2, y: 10, z: 3 };

// A 3 x 3 x 4 grid at (2, 10, 3) with stone and a flower in its two inner cells, (3, 11, 4) and (3, 11, 5), and dirt
// in a border cell.
fn plan(harness: &Harness, options: AmmlMesherOptions) -> Plan {
    let palette = vec![
        handle(harness, ChunkGame::block(ChunkKind::Stone)),
        handle(harness, ChunkGame::block(ChunkKind::Dirt)),
        handle(harness, ChunkGame::block(ChunkKind::Flower)),
    ];
    let mut cells = vec![u16::MAX; 36];
    cells[cell(DIMS, 1, 1, 1)] = 0;
    cells[cell(DIMS, 1, 1, 2)] = 2;
    cells[cell(DIMS, 1, 2, 1)] = 1;
    Plan { dims: DIMS, origin: ORIGIN, cells, palette, options }
}

fn options(alpha: u8, skip: bool) -> AmmlMesherOptions {
    AmmlMesherOptions { alpha, skip_matching_world: u8::from(skip), water_tint: 0, _reserved: 0 }
}

fn render_with(plan: Plan) {
    *PLAN.lock().unwrap() = Some(plan);
    RenderGame::render();
}

fn tessellated() -> Vec<String> {
    MesherGame::log().into_iter().filter(|line| line.starts_with("tessellate")).collect()
}

fn constructions() -> usize {
    MesherGame::log().iter().filter(|line| line.starts_with("ctor")).count()
}

#[test]
fn build_tessellates_each_inner_cell_into_a_ghost_mesh() {
    let harness = start();
    join(&harness);

    render_with(plan(&harness, options(0x80, false)));

    let built = builds();
    assert_eq!(built.len(), 1);
    assert_eq!(built[0].status, AmmlStatus::OK, "{built:?}");
    // One quad each for stone and the flower, and the back face of the flower (a crossed plant).
    assert_eq!(built[0].out.quads, 3);
    assert_ne!(built[0].out.mesh.id, 0);
    assert!(built[0].skipped.is_empty());
    let log = MesherGame::log();
    // The air (`minecraft:air`, the game's hash) and the grass block come from the registry before the tessellator is
    // made.
    assert_eq!(log[0], "lookup minecraft:air hash=0xbd584baf00003448");
    assert!(log[1].starts_with("lookup minecraft:grass_block hash="), "{}", log[1]);
    assert_eq!(log[2..4], ["ctor source=true".to_string(), "set region source=true".to_string()]);
    // The border cell is in the window but not drawn; the cell above stone is the dirt of the border.
    assert_eq!(
        tessellated(),
        [
            "tessellate stone pos=3,11,4 layer=0 cached=1 normals=0 ao=0 window=stone above=dirt extra=air air0=air",
            "tessellate flower pos=3,11,5 layer=0 cached=1 normals=0 ao=0 window=flower above=air extra=air air0=air",
        ]
    );
    // The positions are relative to the grid origin, each face pushed 0.01 into its cell; every vertex has the alpha.
    let (positions, colors, aabb) = MeshGame::last_end().unwrap();
    assert_eq!(positions.len(), 12);
    assert!((positions[0][1] - 1.99).abs() < 1e-5, "{positions:?}");
    assert_eq!(positions[0][0], 1.0);
    assert!(colors.iter().all(|color| color >> 24 == 0x80), "{colors:x?}");
    assert_eq!(aabb[0][0], 1.0);
    assert_eq!(aabb[1][2], 3.0);
    let end = MeshGame::log().into_iter().find(|line| line.starts_with("end ")).unwrap();
    assert!(end.starts_with("end upload=0 vertices=12 count=12 "), "{end}");
}

#[test]
fn empty_cells_and_extra_blocks_are_air_not_the_world() {
    let harness = start();
    join(&harness);
    // The world has a slab above the flower cell and water in it; the game's cache reads its placeholder cells there.
    ChunkGame::set_block(3, 5, 12, ChunkKind::Slab);
    ChunkGame::set_block(3, 5, 11, ChunkKind::Water);
    let mut with_air = plan(&harness, options(0xFF, false));
    // A palette entry that is the air block is not drawn.
    with_air.palette.push(handle(&harness, ChunkGame::air()));
    with_air.cells[cell(DIMS, 1, 1, 1)] = 3;

    render_with(with_air);

    assert_eq!(builds()[0].status, AmmlStatus::OK);
    assert_eq!(
        tessellated(),
        ["tessellate flower pos=3,11,5 layer=0 cached=1 normals=0 ao=0 window=flower above=air extra=air air0=air"]
    );
}

#[test]
fn every_render_layer_of_a_block_is_a_pass() {
    let harness = start();
    join(&harness);
    let mut water = plan(&harness, options(0xFF, false));
    water.palette[0] = handle(&harness, ChunkGame::block(ChunkKind::Water));
    water.cells[cell(DIMS, 1, 1, 2)] = u16::MAX;

    render_with(water);

    assert_eq!(builds()[0].out.quads, 2);
    let layers: Vec<String> = tessellated().iter().map(|line| line.split(' ').nth(3).unwrap().to_string()).collect();
    assert_eq!(layers, ["layer=0", "layer=3"]);
}

#[test]
fn grid_over_the_window_is_refused() {
    let harness = start();
    join(&harness);
    let mut wide = plan(&harness, options(0xFF, false));
    wide.dims = [21, 1, 1];
    wide.cells = vec![u16::MAX; 21];

    render_with(wide);

    assert_eq!(builds()[0].status, AmmlStatus::REFUSED);
    assert_eq!(MesherGame::log(), Vec::<String>::new());
}

#[test]
fn cells_that_do_not_match_the_dims_are_an_invalid_argument() {
    let harness = start();
    join(&harness);
    let mut short = plan(&harness, options(0xFF, false));
    short.cells.pop();

    render_with(short);

    assert_eq!(builds()[0].status, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(MesherGame::log(), Vec::<String>::new());
}

#[test]
fn stale_palette_handle_is_stale() {
    let harness = start();
    join(&harness);
    let stale = plan(&harness, options(0xFF, false));
    harness.state.resources().remove(ResourceId(stale.palette[2].0)).unwrap();

    render_with(stale);

    assert_eq!(builds()[0].status, AmmlStatus::STALE);
    assert!(tessellated().is_empty());
}

#[test]
fn cell_past_the_palette_is_invalid_argument() {
    let harness = start();
    join(&harness);
    let mut past = plan(&harness, options(0xFF, false));
    past.cells[cell(DIMS, 1, 1, 1)] = 3;

    render_with(past);

    assert_eq!(builds()[0].status, AmmlStatus::INVALID_ARGUMENT);
    assert!(tessellated().is_empty());
}

#[test]
fn skip_matching_world_lists_the_skipped_cells() {
    let harness = start();
    join(&harness);
    // The world holds stone at the stone cell (3, 11, 4) and air at the flower cell.
    ChunkGame::set_block(3, 4, 11, ChunkKind::Stone);

    render_with(plan(&harness, options(0xFF, true)));

    let built = builds();
    assert_eq!(built[0].status, AmmlStatus::OK);
    assert_eq!(built[0].skipped, vec![cell(DIMS, 1, 1, 1) as u32]);
    assert_eq!(built[0].out.skipped_count, 1);
    // The skipped cell stays in the window, so its neighbours join it; only the flower is drawn.
    assert_eq!(tessellated().len(), 1);
    assert!(tessellated()[0].starts_with("tessellate flower"), "{:?}", tessellated());
    assert_eq!(built[0].out.quads, 2);
}

#[test]
fn a_build_with_nothing_to_draw_gives_no_mesh() {
    let harness = start();
    join(&harness);
    let mut empty = plan(&harness, options(0xFF, false));
    empty.cells = vec![u16::MAX; 36];

    render_with(empty);

    let built = builds();
    assert_eq!(built[0].status, AmmlStatus::OK);
    assert_eq!((built[0].out.mesh.id, built[0].out.quads), (0, 0));
    // No list was opened; the id names no mesh, so a draw of it is stale.
    assert!(!MeshGame::tessellating());
    assert_eq!(built[0].draw, AmmlStatus::STALE);
}

#[test]
fn tessellator_is_made_once_per_world() {
    let harness = start();
    join(&harness);
    let first = plan(&harness, options(0xFF, false));

    render_with(first.clone());
    render_with(first);
    assert_eq!(constructions(), 1);
    // Each build points the tessellator at the block source of the local player again.
    assert_eq!(MesherGame::log().iter().filter(|line| line.starts_with("set region")).count(), 2);

    SessionGame::exit();

    // The tessellator is destroyed at the world leave, once, and its game heap block freed.
    assert_eq!(MesherGame::log().iter().filter(|line| *line == "dtor").count(), 1);
    assert_eq!(HeapProbe::freed().iter().filter(|(_, size)| *size == TESSELLATOR_SIZE).count(), 1);

    SessionGame::start_join(&crate::support::CLIENT_A, [0; 7]);
    join(&harness);
    render_with(plan(&harness, options(0xFF, false)));

    assert_eq!(constructions(), 2);
    assert_eq!(builds().iter().filter(|built| built.status == AmmlStatus::OK).count(), 3);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "mesher build time");
    let thread = r#"world thread {"hook":"BlockTessellator.tessellateInWorld","game":true}"#;
    assert!(log.contains(thread), "{log}");
}

#[test]
fn outside_the_dispatch_is_refused() {
    let harness = start();
    join(&harness);
    let plan = plan(&harness, options(0xFF, false));

    let built = mesher_build(1, &plan);

    assert_eq!(built.status, AmmlStatus::REFUSED);
    assert_eq!(MesherGame::log(), Vec::<String>::new());
}

// A plan of one inner cell, (1, 1, 1) = the world position (3, 11, 4), of the stand-in block `kind`.
fn single(harness: &Harness, kind: ChunkKind, options: AmmlMesherOptions) -> Plan {
    let mut cells = vec![u16::MAX; 36];
    cells[cell(DIMS, 1, 1, 1)] = 0;
    Plan { dims: DIMS, origin: ORIGIN, cells, palette: vec![handle(harness, ChunkGame::block(kind))], options }
}

fn with_water_tint(alpha: u8) -> AmmlMesherOptions {
    AmmlMesherOptions { alpha, skip_matching_world: 0, water_tint: 1, _reserved: 0 }
}

// The vertex colours of the last mesh the stand-in tessellator ended.
fn end_colors() -> Vec<u32> {
    MeshGame::last_end().unwrap().1
}

fn policy_lines() -> Vec<String> {
    MesherGame::log().into_iter().filter(|line| line.starts_with("policy")).collect()
}

fn water_lines() -> Vec<String> {
    MesherGame::log().into_iter().filter(|line| line.starts_with("water color") || line.starts_with("biome")).collect()
}

#[test]
fn tint_method_none_writes_no_colour() {
    let harness = start();
    join(&harness);

    render_with(plan(&harness, options(0x80, false)));

    // Stone and the flower have no tint method: the game's policy is never asked and the colours stay the game's own.
    assert!(policy_lines().is_empty(), "{:?}", policy_lines());
    assert!(water_lines().is_empty(), "{:?}", water_lines());
    assert!(end_colors().iter().all(|color| *color == 0x80FF_FFFF), "{:x?}", end_colors());
}

#[test]
fn a_tinted_block_takes_the_policy_colour_at_its_position() {
    let harness = start();
    join(&harness);
    MesherGame::set_tint(ChunkKind::Stone, 3);

    render_with(single(&harness, ChunkKind::Stone, options(0x80, false)));

    // The game's policy of the method, for the block, the world position of the cell and no tint cache.
    assert_eq!(policy_lines(), ["policy get method=3 block=stone pos=3,11,4 region=true cache=null"]);
    // (0.5, 1, 0.25) on a grey vertex, the alpha kept.
    assert!(end_colors().iter().all(|color| *color == 0x8040_FF80), "{:x?}", end_colors());
}

#[test]
fn a_tint_method_past_the_policy_table_is_no_tint() {
    let harness = start();
    join(&harness);
    // The game's table has 9 entries; a larger value reads the plain policy (`cmp cl,9; cmovae`).
    MesherGame::set_tint(ChunkKind::Stone, 9);

    render_with(single(&harness, ChunkKind::Stone, options(0x80, false)));

    assert!(policy_lines().is_empty());
    assert!(end_colors().iter().all(|color| *color == 0x80FF_FFFF), "{:x?}", end_colors());
}

#[test]
fn the_grass_block_is_tinted_on_its_top_face() {
    let harness = start();
    join(&harness);
    // The stand-in registry gives the dirt as `minecraft:grass_block`.
    MesherGame::set_tint(ChunkKind::Dirt, 5);

    render_with(single(&harness, ChunkKind::Dirt, options(0xFF, false)));

    assert_eq!(policy_lines(), ["policy get method=5 block=dirt pos=3,11,4 region=true cache=null"]);
    // The stand-in tessellator writes the top face only: it is up, so it takes the colour.
    assert!(end_colors().iter().all(|color| *color == 0xFF40_FF80), "{:x?}", end_colors());
}

#[test]
fn water_tint_off_leaves_water_grey() {
    let harness = start();
    join(&harness);
    MesherGame::set_tint(ChunkKind::Water, 6);

    render_with(single(&harness, ChunkKind::Water, options(0xFF, false)));

    // No biome is read, the game's policy is not asked for water, and both layer passes stay the texture's grey.
    assert!(water_lines().is_empty(), "{:?}", water_lines());
    assert!(policy_lines().is_empty(), "{:?}", policy_lines());
    assert_eq!(end_colors().len(), 8);
    assert!(end_colors().iter().all(|color| *color == 0xFFFF_FFFF), "{:x?}", end_colors());
}

#[test]
fn water_tint_on_gives_water_the_biome_colour() {
    let harness = start();
    join(&harness);
    MesherGame::set_tint(ChunkKind::Water, 6);

    render_with(single(&harness, ChunkKind::Water, with_water_tint(0xFF)));

    // The chunk of the cell, its biome at the column (3, 4) and the y 75 above the chunk's lowest block (-64), then the
    // biome's water colour at the world position; once for each of the two layer passes.
    let lines = water_lines();
    assert_eq!(lines.len(), 4, "{lines:?}");
    assert_eq!(lines[0], "biome column=3,4 y=75");
    assert_eq!(lines[1], "water color pos=3,11,4");
    assert!(policy_lines().is_empty());
    // 0xFF204060 is alpha, red 20h, green 40h, blue 60h; the packed vertex colour is alpha, blue, green, red.
    assert!(end_colors().iter().all(|color| *color == 0xFF60_4020), "{:x?}", end_colors());
}

#[test]
fn water_tint_without_the_area_leaves_water_grey() {
    let harness = start_without(&["BiomeColorSampling.getWaterColor"]);
    join(&harness);
    MesherGame::set_tint(ChunkKind::Water, 6);

    render_with(single(&harness, ChunkKind::Water, with_water_tint(0xFF)));

    // The mesher is still offered; the option is ignored without the area.
    assert_eq!(builds()[0].status, AmmlStatus::OK);
    assert!(water_lines().is_empty(), "{:?}", water_lines());
    assert!(end_colors().iter().all(|color| *color == 0xFFFF_FFFF), "{:x?}", end_colors());
}

// Runs `body` at the next dispatch and gives what it returned.
fn dispatch<R: Send + 'static>(body: impl Fn(u64) -> R + Send + 'static) -> R {
    let slot = Arc::new(Mutex::new(None));
    let put = slot.clone();
    *ACTION.lock().unwrap() = Some(Box::new(move |frame| *put.lock().unwrap() = Some(body(frame))));
    RenderGame::render();
    *ACTION.lock().unwrap() = None;
    let result = slot.lock().unwrap().take();
    result.unwrap()
}

fn group_begin(frame: u64) -> (AmmlStatus, AmmlMeshGroup) {
    let slot = [16u32, 16, 16];
    let mut group = AmmlMeshGroup::default();
    // SAFETY: the table is live; the module, the slot size and the output live for the call.
    let status = unsafe {
        (table().mesher_group_begin.unwrap())(AmmlStr::from_str(MODULE), frame, &raw const slot, &raw mut group)
    };
    (status, group)
}

fn group_append(frame: u64, group: AmmlMeshGroup, plan: &Plan, offset: AmmlBlockPos) -> AmmlStatus {
    let grid = AmmlBlockGrid {
        dims: plan.dims,
        origin: plan.origin,
        cells: plan.cells.as_ptr(),
        cell_count: plan.cells.len() as u32,
        palette: plan.palette.as_ptr(),
        palette_len: plan.palette.len() as u32,
    };
    // SAFETY: the table is live; the module, the grid with its cells and palette, the offset and the options live for
    // the call.
    unsafe {
        (table().mesher_group_append.unwrap())(
            AmmlStr::from_str(MODULE),
            frame,
            group,
            &raw const grid,
            &raw const offset,
            &raw const plan.options,
        )
    }
}

fn group_finish(frame: u64, group: AmmlMeshGroup) -> (AmmlStatus, AmmlMeshBuilt) {
    let mut out = AmmlMeshBuilt::default();
    // SAFETY: the table is live; the module and the output live for the call.
    let status =
        unsafe { (table().mesher_group_finish.unwrap())(AmmlStr::from_str(MODULE), frame, group, &raw mut out) };
    (status, out)
}

fn group_discard(group: AmmlMeshGroup) -> AmmlStatus {
    // SAFETY: the table is live; the module lives for the call.
    unsafe { (table().mesher_group_discard.unwrap())(AmmlStr::from_str(MODULE), group) }
}

const NO_OFFSET: AmmlBlockPos = AmmlBlockPos { x: 0, y: 0, z: 0 };

fn data_lines(prefix: &str) -> Vec<String> {
    MesherGame::log().into_iter().filter(|line| line.starts_with(prefix)).collect()
}

#[test]
fn append_offsets_the_grid() {
    let harness = start();
    join(&harness);
    let grid = plan(&harness, options(0xFF, false));
    let offset = AmmlBlockPos { x: 10, y: 20, z: 30 };

    let (began, appended, finished, built) = dispatch(move |frame| {
        let (began, group) = group_begin(frame);
        let appended = group_append(frame, group, &grid, offset);
        let (finished, built) = group_finish(frame, group);
        (began, appended, finished, built)
    });

    assert_eq!((began, appended, finished), (AmmlStatus::OK, AmmlStatus::OK, AmmlStatus::OK));
    assert_eq!((built.quads, built.skipped_count), (3, 0));
    assert_ne!(built.mesh.id, 0);
    // The grid's own build puts the stone's top face at (1, 1.99, 1) from the grid origin; the group puts the grid at the
    // offset, so every position is that plus (10, 20, 30).
    let (positions, _, aabb) = MeshGame::last_end().unwrap();
    assert_eq!(positions.len(), 12);
    assert!((positions[0][1] - 21.99).abs() < 1e-4, "{positions:?}");
    assert_eq!((positions[0][0], positions[0][2]), (11.0, 31.0));
    assert_eq!((aabb[0][0], aabb[1][2]), (11.0, 33.0));
    // The cells are tessellated at their world positions, as a build does: the offset moves the output only.
    assert!(tessellated()[0].starts_with("tessellate stone pos=3,11,4"), "{:?}", tessellated());
}

#[test]
fn finish_makes_one_mesh_of_every_append() {
    let harness = start();
    join(&harness);
    let first = plan(&harness, options(0xFF, false));
    let second = single(&harness, ChunkKind::Stone, options(0xFF, false));

    let (finished, built, again) = dispatch(move |frame| {
        let (_, group) = group_begin(frame);
        assert_eq!(group_append(frame, group, &first, NO_OFFSET), AmmlStatus::OK);
        assert_eq!(group_append(frame, group, &second, AmmlBlockPos { x: 5, y: 0, z: 0 }), AmmlStatus::OK);
        let (finished, built) = group_finish(frame, group);
        // The finish consumed the group.
        let (again, _) = group_finish(frame, group);
        (finished, built, again)
    });

    assert_eq!(finished, AmmlStatus::OK);
    // Three quads of the first grid and the stone's one quad of the second.
    assert_eq!(built.quads, 4);
    assert_ne!(built.mesh.id, 0);
    assert_eq!(again, AmmlStatus::STALE);
    // The merged vertices became a triangle list in the frame's tessellator, ended once into a mesh that is uploaded.
    let begins = MeshGame::log().into_iter().filter(|line| line.starts_with("begin")).collect::<Vec<_>>();
    assert_eq!(begins.len(), 3, "{begins:?}");
    assert!(begins[0].contains("mode=1") && begins[1].contains("mode=1"), "{begins:?}");
    assert!(begins[2].contains("mode=2"), "{begins:?}");
    assert_eq!(data_lines("mesh data move"), ["mesh data move vertices=16"]);
    let ends = MeshGame::log().into_iter().filter(|line| line.starts_with("end upload=0")).collect::<Vec<_>>();
    assert_eq!(ends.len(), 1, "{ends:?}");
    assert!(ends[0].starts_with("end upload=0 vertices=16 count=16 "), "{ends:?}");
    // The finished group's buffer was moved out: it is destroyed empty, once.
    assert_eq!(data_lines("mesh data dtor"), ["mesh data dtor vertices=0"]);
    assert!(!MeshGame::tessellating());
}

#[test]
fn group_is_stale_after_world_left() {
    let harness = start();
    join(&harness);
    let grid = plan(&harness, options(0xFF, false));
    let kept = grid.clone();
    let (_, group) = dispatch(move |frame| {
        let (status, group) = group_begin(frame);
        assert_eq!(group_append(frame, group, &grid, NO_OFFSET), AmmlStatus::OK);
        (status, group)
    });
    assert!(data_lines("mesh data dtor").is_empty());

    SessionGame::exit();

    // The buffer is destroyed at the world leave, on the game thread, with its vertices.
    assert_eq!(data_lines("mesh data dtor"), ["mesh data dtor vertices=12"]);
    SessionGame::start_join(&crate::support::CLIENT_A, [0; 7]);
    join(&harness);
    let (appended, finished) =
        dispatch(move |frame| (group_append(frame, group, &kept, NO_OFFSET), group_finish(frame, group).0));
    assert_eq!((appended, finished), (AmmlStatus::STALE, AmmlStatus::STALE));
    // Never destroyed twice.
    assert_eq!(data_lines("mesh data dtor").len(), 1);
}

#[test]
fn group_calls_outside_the_dispatch_are_refused() {
    let harness = start();
    join(&harness);
    let grid = plan(&harness, options(0xFF, false));

    let (began, group) = group_begin(1);
    let appended = group_append(1, AmmlMeshGroup { id: 1, generation: 0 }, &grid, NO_OFFSET);
    let (finished, _) = group_finish(1, AmmlMeshGroup { id: 1, generation: 0 });

    assert_eq!((began, appended, finished), (AmmlStatus::REFUSED, AmmlStatus::REFUSED, AmmlStatus::REFUSED));
    assert_eq!(group, AmmlMeshGroup::default());
    assert!(data_lines("mesh data").is_empty());
}

#[test]
fn a_failed_append_leaves_the_group_as_it_was() {
    let harness = start();
    join(&harness);
    let good = plan(&harness, options(0xFF, false));
    let mut stale = good.clone();
    harness.state.resources().remove(ResourceId(stale.palette[2].0)).unwrap();
    stale.cells[cell(DIMS, 1, 1, 2)] = 2;

    let (refused, finished, built) = dispatch(move |frame| {
        let (_, group) = group_begin(frame);
        let refused = group_append(frame, group, &stale, NO_OFFSET);
        let (finished, built) = group_finish(frame, group);
        (refused, finished, built)
    });

    // The stale palette handle is refused before any cell is tessellated; the group is still open and empty.
    assert_eq!(refused, AmmlStatus::STALE);
    assert_eq!(finished, AmmlStatus::OK);
    assert_eq!((built.mesh.id, built.quads), (0, 0));
    assert!(tessellated().is_empty());
}

#[test]
fn a_mod_holds_at_most_64_groups() {
    let harness = start();
    join(&harness);

    let statuses = dispatch(|frame| (0..65).map(|_| group_begin(frame).0).collect::<Vec<_>>());

    assert!(statuses[..64].iter().all(|status| *status == AmmlStatus::OK));
    assert_eq!(statuses[64], AmmlStatus::REFUSED);
}

#[test]
fn a_group_slot_of_no_size_is_an_invalid_argument() {
    let harness = start();
    join(&harness);

    let statuses = dispatch(|frame| {
        let mut group = AmmlMeshGroup::default();
        let flat = [16u32, 0, 16];
        // SAFETY: the table is live; the module, the slot size and the output live for the call.
        let flat_status = unsafe {
            (table().mesher_group_begin.unwrap())(AmmlStr::from_str(MODULE), frame, &raw const flat, &raw mut group)
        };
        // SAFETY: as above; a null slot size is checked before it is read.
        let null_status = unsafe {
            (table().mesher_group_begin.unwrap())(AmmlStr::from_str(MODULE), frame, std::ptr::null(), &raw mut group)
        };
        (flat_status, null_status)
    });

    assert_eq!(statuses, (AmmlStatus::INVALID_ARGUMENT, AmmlStatus::INVALID_ARGUMENT));
}

#[test]
fn a_discarded_group_gives_its_slot_back_and_its_buffer_is_destroyed_once() {
    let harness = start();
    join(&harness);
    let grid = plan(&harness, options(0xFF, false));

    // 65 groups begun and discarded, one at a time: the slot comes back each time.
    let (statuses, last) = dispatch(move |frame| {
        let mut statuses = Vec::new();
        let mut last = AmmlMeshGroup::default();
        for _ in 0..65 {
            let (began, group) = group_begin(frame);
            statuses.push(began);
            statuses.push(group_discard(group));
            last = group;
        }
        let (_, filled) = group_begin(frame);
        assert_eq!(group_append(frame, filled, &grid, NO_OFFSET), AmmlStatus::OK);
        statuses.push(group_discard(filled));
        (statuses, last)
    });
    assert!(statuses.iter().all(|status| *status == AmmlStatus::OK), "{statuses:?}");
    // A discarded group is stale; a second discard frees nothing.
    assert_eq!(group_discard(last), AmmlStatus::STALE);

    // The buffers are destroyed on the `renderLevel` thread at its next call, each once.
    RenderGame::render();
    let dtors = data_lines("mesh data dtor");
    assert_eq!(dtors.len(), 66, "{dtors:?}");
    assert_eq!(dtors.iter().filter(|line| *line == "mesh data dtor vertices=12").count(), 1, "{dtors:?}");
    RenderGame::render();
    assert_eq!(data_lines("mesh data dtor").len(), 66);
}
