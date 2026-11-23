use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml::{
    ActorAdapter, BlockEntityAdapter, BlockEntityDetours, BlockHandles, ClientAdapter, ModListener, ModLoader,
    SessionAdapter,
};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlBlockEntityCamera, AmmlBlockEntityGhost, AmmlBlockHandle, AmmlBlockPos, AmmlEventContext,
    AmmlEventHeader, AmmlFace, AmmlGhostEntityOptions, AmmlRenderTable, AmmlServices, AmmlSkullPose, AmmlStatus,
    AmmlStr, BlockEntityPass,
};
use amml_data::AdapterCatalog;
use amml_runtime::{ModRegistry, SkullTransform};

use crate::support::{
    ActorGame, BlockEntityGame, ClientGame, EventsMod, GAME_CLIENT_A, Harness, RenderGame, SessionGame,
};

// The module of the mod that `start` loads; its mod index is 0.
const MODULE: &str = "@events/events";
const OWNER: usize = 0;
const PASS: &str = "render.block_entity_pass";
const ENGAGED_63: u64 = 0x38;

// The address of the loader's render table.
static TABLE: AtomicUsize = AtomicUsize::new(0);
// What the pass listener saw, and what it runs at each dispatch with the frame id.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
type Action = Box<dyn Fn(u64) + Send>;
static ACTION: Mutex<Option<Action>> = Mutex::new(None);

fn table() -> AmmlRenderTable {
    // SAFETY: `TABLE` holds the address of the loader's render table, which lives for the process.
    unsafe { AmmlRenderTable::read(std::ptr::with_exposed_provenance(TABLE.load(Ordering::SeqCst))) }.unwrap()
}

unsafe extern "C" fn on_pass(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `render.block_entity_pass` with a `BlockEntityPass`.
    let event = unsafe { &*event.cast::<BlockEntityPass>() };
    SEEN.lock().unwrap().push(format!("pass camera={}", event.camera.0));
    if let Some(action) = ACTION.lock().unwrap().as_ref() {
        action(event.frame_id);
    }
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

fn on_each_pass(action: impl Fn(u64) + Send + 'static) {
    *ACTION.lock().unwrap() = Some(Box::new(action));
}

// The adapters over the stand-ins (the 26.63 factory, or the 70.26 one with `flag_form`), one loaded mod and the render
// table found. No listener yet.
fn install(without: &[&str], flag_form: bool, engaged: u64) -> (Harness, Vec<amml_data::Availability>) {
    let harness = Harness::start_with_entries(&BlockEntityGame::entries(without, flag_form, engaged));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    let rows = BlockEntityAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let render = unsafe { AmmlServices::render(services) }.unwrap();
    TABLE.store(render.expose_provenance(), Ordering::SeqCst);
    (harness, rows)
}

fn listen(harness: &Harness) -> amml_runtime::ListenerId {
    let listener = ModListener { owner: OWNER, callback: on_pass, user: 0 };
    harness.state.events().listen(PASS, 1, AmmlBand::NORMAL, listener).unwrap()
}

// `install` with every entry, a pass listener, then a joined world whose local player is the stand-in player.
fn start_with(flag_form: bool, engaged: u64) -> Harness {
    let (harness, _) = install(&[], flag_form, engaged);
    listen(&harness);
    join(&harness);
    harness
}

fn start() -> Harness {
    start_with(false, ENGAGED_63)
}

fn join(harness: &Harness) {
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(ActorGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    assert_ne!(harness.state.events().gate().epoch().0, 0);
    RenderGame::set_camera(Some([0.0, 70.0, 0.0]));
}

fn handle(harness: &Harness, address: usize) -> AmmlBlockHandle {
    let epoch = harness.state.events().gate().epoch();
    let id = harness.state.resources().insert(BlockHandles::BLOCK, OWNER, epoch, address).unwrap();
    AmmlBlockHandle(id.0)
}

fn create(block: AmmlBlockHandle, nbt: &[u8], options: AmmlGhostEntityOptions) -> (AmmlStatus, AmmlBlockEntityGhost) {
    let mut out = AmmlBlockEntityGhost::default();
    // SAFETY: the table is live; the module, the bytes, the options and the output live for the call.
    let status = unsafe {
        (table().block_entity_create.unwrap())(
            AmmlStr::from_str(MODULE),
            block,
            if nbt.is_empty() { std::ptr::null() } else { nbt.as_ptr() },
            nbt.len(),
            &raw const options,
            &raw mut out,
        )
    };
    (status, out)
}

fn draw(frame: u64, ghost: AmmlBlockEntityGhost, pos: AmmlBlockPos) -> AmmlStatus {
    // SAFETY: the table is live; the module and the position live for the call.
    unsafe { (table().block_entity_draw.unwrap())(AmmlStr::from_str(MODULE), frame, ghost, &raw const pos) }
}

fn free(ghost: AmmlBlockEntityGhost) -> AmmlStatus {
    // SAFETY: the table is live; the module lives for the call.
    unsafe { (table().block_entity_free.unwrap())(AmmlStr::from_str(MODULE), ghost) }
}

fn chest_pair(x: i32, z: i32) -> AmmlGhostEntityOptions {
    AmmlGhostEntityOptions { has_chest_pair: 1, chest_pair: AmmlBlockPos { x, y: 0, z }, ..Default::default() }
}

fn skull(face: AmmlFace, turn: u8) -> AmmlGhostEntityOptions {
    AmmlGhostEntityOptions { has_skull: 1, skull: AmmlSkullPose { face, turn }, ..Default::default() }
}

fn calls(prefix: &str) -> Vec<String> {
    BlockEntityGame::log().into_iter().filter(|line| line.starts_with(prefix)).collect()
}

#[test]
fn camera_is_player_for_the_player_renderer() {
    let _harness = start();
    let context = BlockEntityGame::context();

    BlockEntityGame::pass(BlockEntityGame::player_renderer(), context, false);
    BlockEntityGame::pass(BlockEntityGame::other_renderer(), context, false);
    // The alpha pass raises nothing.
    BlockEntityGame::pass(BlockEntityGame::player_renderer(), context, true);

    assert_eq!(
        seen(),
        [
            format!("pass camera={}", AmmlBlockEntityCamera::PLAYER.0),
            format!("pass camera={}", AmmlBlockEntityCamera::OTHER.0),
        ]
    );
    // The game's own body ran first, every time.
    assert_eq!(calls("body"), ["body alpha=0", "body alpha=0", "body alpha=1"]);
}

#[test]
fn chest_pair_needs_a_chest() {
    let harness = start();
    let chest = handle(&harness, BlockEntityGame::chest_block());
    let sign = handle(&harness, BlockEntityGame::sign_block());

    let (made, _) = create(chest, &[], chest_pair(0, 1));
    let (refused, _) = create(sign, &[], chest_pair(1, 0));
    let (diagonal, _) = create(chest, &[], chest_pair(1, 1));

    assert_eq!(made, AmmlStatus::OK);
    assert_eq!(refused, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(diagonal, AmmlStatus::INVALID_ARGUMENT);
    let actors = BlockEntityGame::actors();
    // The chest became the lead of a pair along Z: the partner pointer is not null, the partner is one block along +z,
    // the axis bit is set and the other flag bits stay.
    assert_ne!(actors[0].paired, 0);
    assert_eq!(actors[0].paired_position, [0, 0, 1]);
    assert_eq!(actors[0].flags, 0x5 | 0x2);
    // The sign was made, found not to be a chest by the game's own type test, and released at once.
    assert_eq!(actors.len(), 2);
    assert_eq!(actors[1].paired, 0);
    assert_eq!(calls("is_type"), ["is_type actor=0 type=2", "is_type actor=1 type=2"]);
    assert_eq!(calls("release"), ["release actor=1"]);
}

#[test]
fn a_paired_chest_ghost_is_unpaired_before_its_release() {
    let harness = start();
    let chest = handle(&harness, BlockEntityGame::chest_block());
    let (made, paired) = create(chest, &[], chest_pair(1, 0));
    let (_, plain) = create(chest, &[], AmmlGhostEntityOptions::default());
    assert_eq!(made, AmmlStatus::OK);
    assert_ne!(BlockEntityGame::actors()[0].paired, 0);

    assert_eq!(free(paired), AmmlStatus::OK);
    assert_eq!(free(plain), AmmlStatus::OK);
    RenderGame::render();

    // The partner pointer (the actor's own address) is 0 when the game's destructor runs; the unpaired chest is left
    // as the game made it.
    assert_eq!(
        calls("paired at release"),
        ["paired at release actor=0 paired=false", "paired at release actor=1 paired=false"]
    );
    assert_eq!(calls("release actor"), ["release actor=0", "release actor=1"]);
}

#[test]
fn chest_pair_needs_its_area() {
    let (harness, rows) = install(&["ChestBlockActor.mPairFlags"], false, ENGAGED_63);
    join(&harness);
    let chest = handle(&harness, BlockEntityGame::chest_block());

    let (status, _) = create(chest, &[], chest_pair(1, 0));

    assert_eq!(status, AmmlStatus::NOT_AVAILABLE);
    let row = rows.iter().find(|row| row.id == "render.chest_pair").unwrap();
    assert_eq!(row.shape, None);
    // Nothing was made for a refused option.
    assert!(BlockEntityGame::actors().is_empty());
}

#[test]
fn draw_outside_the_pass_is_refused() {
    let harness = start();
    let ghost = create(handle(&harness, BlockEntityGame::chest_block()), &[], AmmlGhostEntityOptions::default()).1;
    let frames = std::sync::Arc::new(Mutex::new(Vec::new()));
    let record = frames.clone();
    on_each_pass(move |frame| {
        let pos = AmmlBlockPos { x: 3, y: 65, z: -4 };
        record.lock().unwrap().push((frame, draw(frame, ghost, pos), draw(frame + 1, ghost, pos)));
    });

    BlockEntityGame::pass(BlockEntityGame::player_renderer(), BlockEntityGame::context(), false);
    let outside = draw(frames.lock().unwrap()[0].0, ghost, AmmlBlockPos { x: 0, y: 0, z: 0 });

    let frames = frames.lock().unwrap();
    assert_eq!((frames[0].1, frames[0].2), (AmmlStatus::OK, AmmlStatus::REFUSED));
    assert_eq!(outside, AmmlStatus::REFUSED);
    assert_eq!(calls("render").len(), 1);
}

#[test]
fn a_ghost_stands_at_its_draw_position_while_it_is_drawn() {
    let harness = start();
    let chest = handle(&harness, BlockEntityGame::chest_block());
    let (_, single) = create(chest, &[], AmmlGhostEntityOptions::default());
    let (_, paired) = create(chest, &[], chest_pair(0, 1));
    on_each_pass(move |frame| {
        assert_eq!(draw(frame, single, AmmlBlockPos { x: 3, y: 65, z: -4 }), AmmlStatus::OK);
        assert_eq!(draw(frame, single, AmmlBlockPos { x: 10, y: 70, z: 7 }), AmmlStatus::OK);
        assert_eq!(draw(frame, paired, AmmlBlockPos { x: -8, y: 64, z: 2 }), AmmlStatus::OK);
    });

    BlockEntityGame::pass(BlockEntityGame::player_renderer(), BlockEntityGame::context(), false);

    // One actor drawn at two places stands at each in turn; the partner of a paired chest moves with it and stays one
    // block along +z. An unpaired chest keeps its partner position as the game made it.
    assert_eq!(
        calls("actor at render"),
        [
            "actor at render actor=0 position=3,65,-4 paired=0,0,0",
            "actor at render actor=0 position=10,70,7 paired=0,0,0",
            "actor at render actor=1 position=-8,64,2 paired=-8,64,3",
        ]
    );
}

#[test]
fn metadata_is_disengaged_at_the_derived_flag() {
    let harness = start_with(true, 0x30);
    // The level's flag byte is 0 here: the factory must get what the level gives, not a constant.
    BlockEntityGame::set_client_side(0);
    let ghost = create(handle(&harness, BlockEntityGame::chest_block()), &[], AmmlGhostEntityOptions::default()).1;
    on_each_pass(move |frame| {
        assert_eq!(draw(frame, ghost, AmmlBlockPos { x: 3, y: 65, z: -4 }), AmmlStatus::OK);
    });

    BlockEntityGame::pass(BlockEntityGame::player_renderer(), BlockEntityGame::context(), false);

    // The dispatcher got its own context's dispatcher, the local player's block source, the ghost's actor and block,
    // the position relative to the camera target (1, 64, 2), the world position, the opaque pass, an empty material,
    // no texture, no breaking, and a disengaged optional at the derived flag.
    assert_eq!(
        calls("render"),
        ["render dispatcher=true source=true actor=0 block=chest pos=2,1,-6 world=3,65,-4 alpha=0 material=0,0 \
          texture=0 breaking=0 engaged@0x30=0"]
    );
    // The 70.26 factory got the level's flag byte.
    assert_eq!(calls("create"), ["create type=chest pos=0,0,0 flag=0"]);
}

#[test]
fn a_flag_past_the_optional_buffer_keeps_the_area_off() {
    let (_harness, rows) = install(&[], false, 0x200);

    let row = rows.iter().find(|row| row.id == "render.block_entities").unwrap();
    assert_eq!(row.shape, None);
    assert_eq!(row.missing, ["RenderMetadata.engaged"]);
}

#[test]
fn no_listener_reads_one_flags_word() {
    let (harness, _) = install(&[], false, ENGAGED_63);
    join(&harness);
    assert_eq!(BlockEntityDetours::flags(), 0);
    // A camera and a context in the first page, which is never mapped: a read of either would end the test process.
    let unmapped = 0x10;

    BlockEntityGame::pass(unmapped, unmapped, false);

    assert_eq!(calls("body"), ["body alpha=0"]);
    let listener = listen(&harness);
    assert_eq!(BlockEntityDetours::flags(), BlockEntityDetours::LISTENER);
    assert!(harness.state.events().unlisten(listener));
    assert_eq!(BlockEntityDetours::flags(), 0);
    assert!(seen().is_empty());
}

#[test]
fn nbt_loads_through_load_static_and_the_tag_is_freed() {
    let harness = start();
    let chest = handle(&harness, BlockEntityGame::chest_block());

    let (status, _) = create(chest, &[10, 0, 0, 0], AmmlGhostEntityOptions::default());

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(
        BlockEntityGame::log(),
        ["nbt read len=4", "load type=chest pos=0,0,0 level=true tag=4 helper=true", "tag dtor"]
    );
}

#[test]
fn ghost_is_stale_after_world_left_and_freed_on_the_render_thread_once() {
    let harness = start();
    let chest = handle(&harness, BlockEntityGame::chest_block());
    let first = create(chest, &[], AmmlGhostEntityOptions::default()).1;
    let second = create(chest, &[], AmmlGhostEntityOptions::default()).1;

    assert_eq!(free(first), AmmlStatus::OK);
    assert_eq!(free(first), AmmlStatus::STALE);
    // The actor waits for the next `renderLevel`.
    assert!(calls("release").is_empty());
    RenderGame::render();
    assert_eq!(calls("release"), ["release actor=0"]);

    SessionGame::exit();

    // The world leave on the game thread released the other actor at once; nothing is released twice.
    assert_eq!(calls("release"), ["release actor=0", "release actor=1"]);
    assert_eq!(free(second), AmmlStatus::STALE);
    RenderGame::render();
    assert_eq!(calls("release").len(), 2);
    assert_eq!(BlockEntityAdapter::get().unwrap().live_ghosts(), 0);
}

#[test]
fn skull_shape_follows_the_resolved_entries() {
    let (_harness, rows) = install(&[], true, 0x30);

    let row = rows.iter().find(|row| row.id == "render.skull_pose").unwrap();
    assert_eq!(row.shape, Some(AdapterCatalog::SKULL_UPRIGHT_SHAPE));
}

#[test]
fn skull_pose_needs_the_ghost_area() {
    let (harness, rows) = install(&["NbtIo.read"], false, ENGAGED_63);
    join(&harness);

    let row = rows.iter().find(|row| row.id == "render.skull_pose").unwrap();
    assert_eq!(row.shape, None);
    assert_eq!(row.missing, ["render.block_entities"]);
}

#[test]
fn skull_pose_on_a_non_skull_is_invalid_argument() {
    let harness = start();
    let chest = handle(&harness, BlockEntityGame::chest_block());
    let skull_block = handle(&harness, BlockEntityGame::skull_block());

    let (on_chest, _) = create(chest, &[], skull(AmmlFace::UP, 4));
    // A face the game does not number and a turn past 15 are refused before any actor is made.
    let (bad_face, _) = create(skull_block, &[], skull(AmmlFace(6), 0));
    let (bad_turn, _) = create(skull_block, &[], skull(AmmlFace::UP, 16));

    assert_eq!(on_chest, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!((bad_face, bad_turn), (AmmlStatus::INVALID_ARGUMENT, AmmlStatus::INVALID_ARGUMENT));
    // The chest was made, found not to be a skull by the game's own type test (skull type 6), and released at once.
    assert_eq!(BlockEntityGame::actors().len(), 1);
    assert_eq!(calls("is_type"), [format!("is_type actor=0 type={}", BlockEntityGame::SKULL_KIND)]);
    assert_eq!(calls("release"), ["release actor=0"]);
    assert_eq!(BlockEntityAdapter::get().unwrap().live_ghosts(), 0);
}

#[test]
fn on_26_63_render_skull_gets_face_turn_and_name() {
    let (harness, rows) = install(&[], false, ENGAGED_63);
    listen(&harness);
    join(&harness);
    let row = rows.iter().find(|row| row.id == "render.skull_pose").unwrap();
    assert_eq!(row.shape, Some(AdapterCatalog::SKULL_RENDER_SHAPE));
    let block = handle(&harness, BlockEntityGame::skull_block());
    let (floor_status, floor) = create(block, &[], skull(AmmlFace::UP, 4));
    let (wall_status, wall) = create(block, &[], skull(AmmlFace::NORTH, 4));
    assert_eq!((floor_status, wall_status), (AmmlStatus::OK, AmmlStatus::OK));
    on_each_pass(move |frame| {
        let pos = AmmlBlockPos { x: 3, y: 65, z: -4 };
        assert_eq!(draw(frame, floor, pos), AmmlStatus::OK);
        assert_eq!(draw(frame, wall, pos), AmmlStatus::OK);
    });

    BlockEntityGame::pass(BlockEntityGame::player_renderer(), BlockEntityGame::context(), false);

    // The dispatcher's skull renderer, the pass context, an empty `RenderMetadata`, the position relative to the camera
    // target (1, 64, 2), the face, the turn in degrees on the floor (4 sixteenths: 90) and 0 on a wall, the ghost's block
    // type name, no material or texture, no breaking, no animation and no glint. The dispatcher itself draws nothing.
    assert_eq!(
        calls("skull"),
        [
            "skull renderer=true context=true metadata_empty=true pos=2,1,-6 face=1 rotation=90 name=skull material=0 \
             texture=0 breaking=0 animation=0 glint=0",
            "skull renderer=true context=true metadata_empty=true pos=2,1,-6 face=2 rotation=0 name=skull material=0 \
             texture=0 breaking=0 animation=0 glint=0",
        ]
    );
    assert!(calls("render").is_empty());
    assert_eq!(calls("is_type"), ["is_type actor=0 type=6", "is_type actor=1 type=6"]);
}

#[test]
fn on_70_26_upright_draws_inside_the_face_transform() {
    let harness = start_with(true, 0x30);
    let block = handle(&harness, BlockEntityGame::skull_block());
    let floor = create(block, &[], skull(AmmlFace::UP, 4)).1;
    let wall = create(block, &[], skull(AmmlFace::EAST, 0)).1;
    on_each_pass(move |frame| {
        let pos = AmmlBlockPos { x: 3, y: 65, z: -4 };
        assert_eq!(draw(frame, floor, pos), AmmlStatus::OK);
        assert_eq!(draw(frame, wall, pos), AmmlStatus::OK);
    });

    BlockEntityGame::pass(BlockEntityGame::player_renderer(), BlockEntityGame::context(), false);

    // Each draw pushes the camera's world matrix stack, marks it dirty, calls `renderSkullUpright` with the position
    // relative to the camera target and the name, and pops.
    let upright = "upright renderer=true context=true metadata_empty=true pos=2,1,-6 name=skull animation=0 glint=0 \
                   dirty=1";
    let push = "push camera_stack=true flag=1";
    let drawn: Vec<String> = BlockEntityGame::log()
        .into_iter()
        .filter(|line| !line.starts_with("create") && !line.starts_with("is_type"))
        .collect();
    assert_eq!(drawn, ["body alpha=0", push, upright, "pop", push, upright, "pop"]);
    // The pushed top (identity here) times the face and turn transform of `SkullTransform`.
    let pos = [2.0, 1.0, -6.0];
    assert_eq!(BlockEntityGame::upright_tops(), [SkullTransform::matrix(1, 4, pos), SkullTransform::matrix(5, 0, pos)]);
}
