use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml::{
    ActorAdapter, ClientAdapter, GameModeAdapter, ModListener, ModLoader, PickRangeShape, PlayerActionAdapter,
    SessionAdapter, UseBuildDetours,
};
use amml_abi::{
    AmmlActorTable, AmmlApi, AmmlBand, AmmlBlockPos, AmmlEventContext, AmmlEventHeader, AmmlFace, AmmlHitResult,
    AmmlOrigin, AmmlServices, AmmlStatus, AmmlStr, AmmlVec3, BuildBlock,
};
use amml_runtime::ModRegistry;

use crate::support::{BuildGame, CLIENT_A, ClientGame, EventsMod, GAME_CLIENT_A, Harness, LogFile, SessionGame};

const USE: &str = "player.use_pressed";
const BUILD: &str = "player.build_block";
// The module of the events-mod fixture at index 0, and the index of the second one.
const MODULE: &str = "@events/events";
const OTHER_OWNER: usize = 1;

// `<owner> origin=<n> local=<n>` per `BuildBlock` listener call, then what the listeners did, in order.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The actor table, for the listener that builds.
static TABLE: AtomicUsize = AtomicUsize::new(0);

const POS: AmmlBlockPos = AmmlBlockPos { x: 4, y: 70, z: -2 };
const CLICK: AmmlVec3 = AmmlVec3 { x: 4.5, y: 71.0, z: -1.75 };

// Installs the session, client, actor, player action and game mode adapters over every stand-in entry but `without`.
fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&BuildGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    PlayerActionAdapter::install(harness.state);
    GameModeAdapter::install(harness.state);
    TABLE.store(std::ptr::from_ref(table(&harness)).addr(), Ordering::SeqCst);
    BuildGame::reset_hit();
    harness
}

// `install` with every entry, after one frame and a joined world whose local player is the stand-in player.
fn start() -> Harness {
    let harness = install(&[]);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(BuildGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

// Loads and enables the events-mod fixtures `plain` (module `@events/events`, index 0) and `provider` (index 1).
fn enable_mods(harness: &Harness) -> ModLoader {
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let provider = EventsMod::build("provider", &["provider"]);
    harness.install_mod("events", "events-provider", &provider, "events_provider.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    loader.enable_all();
    loader
}

fn table(harness: &Harness) -> &'static AmmlActorTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::actor(services) }.unwrap();
    // SAFETY: as above.
    unsafe { &*table }
}

fn player() -> *const c_void {
    std::ptr::with_exposed_provenance(BuildGame::player())
}

fn build_at_with(table: &AmmlActorTable, module: &str, face: AmmlFace, click: AmmlVec3) -> (AmmlStatus, u8) {
    let mut out = 0xEEu8;
    let pos = POS;
    // SAFETY: the table is live; the module view, the position, the click and `out` live for the call.
    let status = unsafe {
        table.build_at.unwrap()(AmmlStr::from_str(module), player(), &raw const pos, face, &raw const click, &mut out)
    };
    (status, out)
}

fn build_at(table: &AmmlActorTable) -> (AmmlStatus, u8) {
    build_at_with(table, MODULE, AmmlFace::UP, CLICK)
}

// Listens to `BuildBlock` as `owner`; `user` 1 cancels, 2 reports a panic.
fn listen_build(harness: &Harness, owner: usize, user: usize) {
    let listener = ModListener { owner, callback: on_build, user };
    harness.state.events().listen(BUILD, 1, AmmlBand::NORMAL, listener).unwrap();
}

unsafe extern "C" fn on_build(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `player.build_block` with a `BuildBlock` and a live context.
    let (event, context) = unsafe { (&mut *event.cast::<BuildBlock>(), &*context) };
    let hit = BuildGame::hit();
    SEEN.lock().unwrap().push(format!(
        "build origin={} local={} sim={} thread={} hit={}",
        event.header.origin.0, event.is_local_player, event.sim_tick, context.thread.0, hit.kind
    ));
    match user.addr() {
        1 => {
            event.header.cancelled = 1;
            AmmlStatus::OK
        }
        2 => AmmlStatus::PANICKED,
        _ => AmmlStatus::OK,
    }
}

// The `UsePressed` listener of the mod at index 0: builds through the actor table, then records the result.
unsafe extern "C" fn build_on_use(_: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: `TABLE` holds the loader's actor table, which lives for the process.
    let table = unsafe { &*std::ptr::with_exposed_provenance::<AmmlActorTable>(TABLE.load(Ordering::SeqCst)) };
    let (status, built) = build_at(table);
    SEEN.lock().unwrap().push(format!("use built status={} out={built}", status.0));
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

#[test]
fn the_areas_are_offered_and_the_build_area_keeps_the_sim_tick() {
    let harness = install(&[]);
    assert!(harness.state.areas().is_available(GameModeAdapter::BUILD, 1));
    assert!(harness.state.areas().is_available(GameModeAdapter::PICK_RANGE, 1));
    assert!(harness.state.areas().is_available(GameModeAdapter::HIT_RESULT, 1));
    assert_eq!(UseBuildDetours::flags() & UseBuildDetours::SIM_TICK, UseBuildDetours::SIM_TICK);
    assert_eq!(table(&harness).size, AmmlActorTable::SIZE_V6);
}

#[test]
fn build_at_raises_build_block_with_origin_this_mod() {
    let harness = start();
    enable_mods(&harness);
    listen_build(&harness, 0, 0);
    listen_build(&harness, OTHER_OWNER, 0);

    let (status, built) = build_at(table(&harness));

    assert_eq!((status, built), (AmmlStatus::OK, 1));
    let game = format!("{}", amml_abi::AmmlThread::GAME.0);
    assert_eq!(
        seen(),
        [
            format!("build origin={} local=1 sim=0 thread={game} hit=0", AmmlOrigin::THIS_MOD.0),
            format!("build origin={} local=1 sim=0 thread={game} hit=0", AmmlOrigin::OTHER_MOD.0),
        ]
    );
    assert_eq!(UseBuildDetours::reentry().actor(), None);
}

#[test]
fn build_at_inside_a_use_pressed_listener_raises_build_block_once_on_the_same_stack() {
    let harness = start();
    enable_mods(&harness);
    let listener = ModListener { owner: 0, callback: build_on_use, user: 0 };
    harness.state.events().listen(USE, 1, AmmlBand::NORMAL, listener).unwrap();
    listen_build(&harness, 0, 0);
    listen_build(&harness, OTHER_OWNER, 0);

    BuildGame::game_use();

    let game = amml_abi::AmmlThread::GAME.0;
    assert_eq!(
        seen(),
        [
            format!("build origin={} local=1 sim=0 thread={game} hit=0", AmmlOrigin::THIS_MOD.0),
            format!("build origin={} local=1 sim=0 thread={game} hit=0", AmmlOrigin::OTHER_MOD.0),
            "use built status=0 out=1".to_string(),
        ]
    );
    assert_eq!(BuildGame::calls().iter().filter(|call| call.starts_with("build ")).count(), 1);
}

#[test]
fn hit_result_is_restored_after_build_at() {
    let harness = start();
    enable_mods(&harness);
    BuildGame::game_build(BuildGame::game_mode(), POS, 1, 0, 1);
    let before = BuildGame::hit();

    let (status, built) = build_at(table(&harness));

    assert_eq!((status, built), (AmmlStatus::OK, 1));
    let calls = BuildGame::calls();
    let game_mode = BuildGame::game_mode().addr();
    assert!(calls[1].starts_with(&format!("build this={game_mode:#x} pos=4,70,-2 face=")), "{calls:?}");
    assert!(calls[1].ends_with(" bytes=1,0,1 hit=0 1 4,70,-2 4.5,71,-1.75"), "{calls:?}");
    let after = BuildGame::hit();
    assert_eq!(
        (after.kind, after.facing, after.block, after.pos),
        (before.kind, before.facing, before.block, before.pos)
    );
    assert_eq!((after.kind, after.facing, after.block, after.pos), (3, 4, [9, 9, 9], [0.5; 3]));
    assert_eq!(after.weak_control, 0x77);
}

#[test]
fn hit_result_is_restored_when_a_listener_panics_or_cancels() {
    let harness = start();
    enable_mods(&harness);
    listen_build(&harness, OTHER_OWNER, 2);

    let (status, built) = build_at(table(&harness));

    assert_eq!((status, built), (AmmlStatus::OK, 1));
    let after = BuildGame::hit();
    assert_eq!((after.kind, after.facing, after.block, after.pos), (3, 4, [9, 9, 9], [0.5; 3]));
    listen_build(&harness, 0, 1);

    let (status, built) = build_at(table(&harness));

    assert_eq!((status, built), (AmmlStatus::OK, 0));
    let after = BuildGame::hit();
    assert_eq!((after.kind, after.facing, after.block, after.pos), (3, 4, [9, 9, 9], [0.5; 3]));
    assert_eq!(BuildGame::calls().len(), 1);
    assert_eq!(UseBuildDetours::reentry().actor(), None);
}

#[test]
fn build_at_refuses_a_bad_face_a_bad_click_and_an_unknown_mod() {
    let harness = start();
    enable_mods(&harness);
    let table = table(&harness);

    assert_eq!(build_at_with(table, MODULE, AmmlFace(6), CLICK).0, AmmlStatus::INVALID_ARGUMENT);
    let bad = AmmlVec3 { x: f32::NAN, ..CLICK };
    assert_eq!(build_at_with(table, MODULE, AmmlFace::UP, bad).0, AmmlStatus::INVALID_ARGUMENT);
    assert_ne!(build_at_with(table, "@nobody/none", AmmlFace::UP, CLICK).0, AmmlStatus::OK);
    assert!(BuildGame::calls().is_empty());
}

#[test]
fn build_at_off_the_game_thread_is_refused() {
    let harness = start();
    enable_mods(&harness);
    let table = table(&harness);

    let status = std::thread::scope(|scope| scope.spawn(|| build_at(table).0).join().unwrap());

    assert_eq!(status, AmmlStatus::REFUSED);
    assert!(BuildGame::calls().is_empty());
}

#[test]
fn the_build_delay_is_read_and_logged_once_per_world() {
    let harness = start();
    let table = table(&harness);
    let read = || {
        let mut out = 0u32;
        // SAFETY: the table is live and `out` is writable.
        let status = unsafe { table.build_delay.unwrap()(player(), &mut out) };
        (status, out)
    };

    assert_eq!(read(), (AmmlStatus::OK, 200));
    BuildGame::set_delay(250);
    assert_eq!(read(), (AmmlStatus::OK, 250));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "build delay read");
    assert_eq!(log.matches("build delay read").count(), 1, "{log}");
    assert!(log.contains("\"value\":200"), "{log}");
}

fn pick_range(table: &AmmlActorTable) -> (AmmlStatus, f32) {
    let mut out = 0.0f32;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { table.max_pick_range.unwrap()(player(), &mut out) };
    (status, out)
}

#[test]
fn pick_range_shape_follows_the_resolved_entries() {
    let harness = start();

    assert_eq!(GameModeAdapter::get().unwrap().pick_shape(), Some(PickRangeShape::Getter));
    assert_eq!(pick_range(table(&harness)), (AmmlStatus::OK, BuildGame::GETTER_RANGE));
    assert_eq!(BuildGame::calls(), [format!("pick range this={:#x}", BuildGame::game_mode().addr())]);
}

#[test]
fn pick_range_without_the_getter_follows_is_creative() {
    let harness = install(&["GameMode.getMaxPickRange"]);
    ClientGame::frame();
    ClientGame::set_player(BuildGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    let table = table(&harness);

    assert_eq!(GameModeAdapter::get().unwrap().pick_shape(), Some(PickRangeShape::Creative));
    assert_eq!(pick_range(table), (AmmlStatus::OK, 6.7));
    BuildGame::set_creative(true);
    assert_eq!(pick_range(table), (AmmlStatus::OK, 12.0));
    let call = format!("is creative this={:#x}", BuildGame::player());
    assert_eq!(BuildGame::calls(), [call.clone(), call]);
}

#[test]
fn without_the_pick_range_entries_the_area_is_not_offered() {
    let harness = install(&["GameMode.getMaxPickRange", "Player.isCreative"]);
    ClientGame::frame();
    ClientGame::set_player(BuildGame::player());
    ClientGame::update(&GAME_CLIENT_A);

    assert!(!harness.state.areas().is_available(GameModeAdapter::PICK_RANGE, 1));
    assert_eq!(pick_range(table(&harness)).0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn a_local_build_on_the_game_thread_sets_the_sim_tick_build_at_passes() {
    let harness = start();
    enable_mods(&harness);
    listen_build(&harness, 0, 0);
    BuildGame::game_build(BuildGame::game_mode(), POS, 1, 0, 1);
    assert_eq!(UseBuildDetours::last_sim_tick(harness.state.events().gate().epoch().0), 1);

    build_at(table(&harness));

    assert_eq!(seen()[1], format!("build origin={} local=1 sim=1 thread=0 hit=0", AmmlOrigin::THIS_MOD.0));
}

#[test]
fn the_sim_tick_does_not_carry_over_a_world_change() {
    let harness = start();
    enable_mods(&harness);
    listen_build(&harness, 0, 0);
    BuildGame::game_build(BuildGame::game_mode(), POS, 1, 0, 1);
    SessionGame::exit();
    ClientGame::update(&GAME_CLIENT_A);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 2);

    build_at(table(&harness));

    assert_eq!(seen()[1], format!("build origin={} local=1 sim=0 thread=0 hit=0", AmmlOrigin::THIS_MOD.0));
}

fn hit_result(table: &AmmlActorTable) -> (AmmlStatus, AmmlHitResult) {
    let mut out = AmmlHitResult { kind: 0xEE, ..AmmlHitResult::default() };
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.hit_result.unwrap())(player(), &mut out) };
    (status, out)
}

#[test]
fn the_hit_result_reads_the_block_the_entity_or_nothing_from_the_levels_hit_result() {
    let harness = start();
    assert!(harness.state.areas().is_available("player.hit_result", 1));
    let table = table(&harness);

    BuildGame::set_hit(0, 3, [4, 70, -2], [4.5, 70.25, -1.0]);
    let block = hit_result(table);
    BuildGame::set_hit(1, 0, [0, 0, 0], [1.0, 2.0, 3.0]);
    let entity = hit_result(table);
    BuildGame::set_hit(2, 0, [0, 0, 0], [1.0, 2.0, 3.0]);
    let far_entity = hit_result(table);
    BuildGame::set_hit(3, 4, [9, 9, 9], [0.5; 3]);
    let nothing = hit_result(table);
    BuildGame::set_hit(0, 9, [1, 1, 1], [0.0; 3]);
    let bad_face = hit_result(table);

    let point = |x, y, z| AmmlVec3 { x, y, z };
    assert_eq!(
        block,
        (
            AmmlStatus::OK,
            AmmlHitResult {
                kind: AmmlHitResult::BLOCK,
                face: AmmlFace(3),
                _reserved: [0; 2],
                pos: AmmlBlockPos { x: 4, y: 70, z: -2 },
                point: point(4.5, 70.25, -1.0)
            }
        )
    );
    assert_eq!(
        entity,
        (
            AmmlStatus::OK,
            AmmlHitResult { kind: AmmlHitResult::ENTITY, point: point(1.0, 2.0, 3.0), ..AmmlHitResult::default() }
        )
    );
    assert_eq!(far_entity, (AmmlStatus::OK, AmmlHitResult::default()));
    assert_eq!(nothing, (AmmlStatus::OK, AmmlHitResult::default()));
    assert_eq!(bad_face, (AmmlStatus::OK, AmmlHitResult::default()));
}

#[test]
fn the_hit_result_is_refused_for_another_actor_and_off_the_game_thread_and_needs_its_entries() {
    let harness = start();
    let table = table(&harness);
    let mut out = AmmlHitResult::default();
    let other = std::ptr::with_exposed_provenance::<c_void>(0x4444);

    // SAFETY: the table is live and `out` is writable.
    let not_player = unsafe { (table.hit_result.unwrap())(other, &mut out) };
    let off_thread = std::thread::spawn(|| {
        let mut out = AmmlHitResult::default();
        // SAFETY: the table is live for the process and `out` is writable.
        unsafe { (table_static().hit_result.unwrap())(player(), &mut out) }
    })
    .join()
    .unwrap();

    assert_eq!((not_player, off_thread), (AmmlStatus::REFUSED, AmmlStatus::REFUSED));
}

fn table_static() -> &'static AmmlActorTable {
    // SAFETY: `TABLE` holds the address of the loader's actor table, which lives for the process.
    unsafe { &*std::ptr::with_exposed_provenance::<AmmlActorTable>(TABLE.load(Ordering::SeqCst)) }
}

#[test]
fn the_hit_result_area_is_not_offered_without_the_hit_result_slot() {
    let harness = install(&["Level.getHitResult"]);

    assert!(!harness.state.areas().is_available("player.hit_result", 1));
}
