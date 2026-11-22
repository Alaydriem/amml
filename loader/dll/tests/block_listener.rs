use std::ffi::c_void;
use std::sync::Mutex;

use amml::{
    ActorAdapter, BlockListener, BlockReadAdapter, ClientAdapter, LoaderState, ModListener, ModLoader, SessionAdapter,
    WorldAdapter,
};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlBlockHandle, AmmlEventContext, AmmlEventHeader, AmmlServices, AmmlStatus, AmmlStr,
    AmmlWorldTable, BlockChanged,
};
use amml_runtime::{ImageLayout, ListenerId, ModRegistry};

use crate::support::{
    ActorGame, BlockGame, BlockReadGame, ChunkGame, ChunkKind, ClientGame, EchoMod, EventsMod, GAME_CLIENT_A, Harness,
    LogFile, SessionGame,
};

// What the `world.block_changed` listener saw, one line per event.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

unsafe extern "C" fn on_changed(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `world.block_changed` with a `BlockChanged` and a live context.
    let (changed, context) = unsafe { (&*event.cast::<BlockChanged>(), &*context) };
    let name = |id: u32| {
        WorldAdapter::get().and_then(|world| world.block_type_name(id).ok()).unwrap_or_else(|| format!("#{id}"))
    };
    SEEN.lock().unwrap().push(format!(
        "dim={} pos={},{},{} layer={} new={} old={} thread={}",
        changed.dimension,
        changed.pos.x,
        changed.pos.y,
        changed.pos.z,
        changed.layer,
        name(changed.new),
        name(changed.old),
        context.thread.0
    ));
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

// Installs the session, client, actor, world and block listener adapters over every stand-in entry but `without`.
fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&BlockGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    BlockListener::install(harness.state, BlockGame::image);
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

fn listen(harness: &Harness) -> ListenerId {
    let listener = ModListener { owner: 0, callback: on_changed, user: 0 };
    harness.state.events().listen("world.block_changed", 1, AmmlBand::NORMAL, listener).unwrap()
}

// Installed, joined, with a listener, and one tick after it so the listener object is attached; gives the listener
// object the stand-in block source got.
fn attached(harness: &Harness) -> usize {
    join(harness);
    listen(harness);
    ClientGame::update(&GAME_CLIENT_A);
    let added = BlockGame::added();
    assert_eq!(added.len(), 1);
    added[0]
}

fn stone() -> usize {
    ChunkGame::block(ChunkKind::Stone)
}

fn dirt() -> usize {
    ChunkGame::block(ChunkKind::Dirt)
}

#[test]
fn no_listener_means_no_attach() {
    let harness = install(&[]);
    join(&harness);

    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::update(&GAME_CLIENT_A);

    assert!(harness.state.events().is_available("world.block_changed", 1));
    assert_eq!(BlockGame::added(), Vec::<usize>::new());
    assert_eq!(BlockListener::attached_source(), None);
}

#[test]
fn a_listener_attaches_at_the_next_tick_and_the_game_thread_change_is_raised_at_once() {
    let harness = install(&[]);
    BlockGame::set_dimension(7);
    let listener = attached(&harness);
    assert_eq!(BlockListener::attached_source(), Some(ActorGame::block_source()));

    BlockGame::fire_changed(listener, [-3, 64, 17], 0, stone(), dirt());

    assert_eq!(seen(), ["dim=7 pos=-3,64,17 layer=0 new=minecraft:stone old=minecraft:dirt thread=0"]);
}

#[test]
fn a_change_inside_one_type_is_raised() {
    let harness = install(&[]);
    let listener = attached(&harness);

    BlockGame::fire_changed(listener, [1, 2, 3], 1, stone(), stone());

    assert_eq!(seen(), ["dim=0 pos=1,2,3 layer=1 new=minecraft:stone old=minecraft:stone thread=0"]);
}

#[test]
fn listener_vtable_copy_keeps_the_other_slots() {
    let harness = install(&[]);
    let listener = attached(&harness);

    let slots = BlockGame::slots_of(listener);
    let base = BlockGame::base();

    for index in 0..BlockGame::SLOTS {
        if index == BlockGame::DESTROYED_SLOT || index == BlockGame::CHANGED_SLOT {
            assert_ne!(slots[index], base[index], "slot {index} is a loader thunk");
            assert_ne!(slots[index], 0);
        } else {
            assert_eq!(slots[index], base[index], "slot {index}");
        }
    }
    assert_ne!(slots[BlockGame::DESTROYED_SLOT], slots[BlockGame::CHANGED_SLOT]);
}

#[test]
fn detach_off_thread_waits_for_the_game_thread() {
    let harness = install(&[]);
    join(&harness);
    let id = listen(&harness);
    ClientGame::update(&GAME_CLIENT_A);
    let listener = BlockGame::added()[0];

    let removed = std::thread::scope(|scope| scope.spawn(|| harness.state.events().unlisten(id)).join().unwrap());

    assert!(removed);
    assert_eq!(BlockGame::removed(), Vec::<usize>::new());
    assert_eq!(BlockListener::attached_source(), Some(ActorGame::block_source()));
    // The next tick on the game thread takes the listener out of the block source and frees it.
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(BlockGame::removed(), [listener]);
    assert_eq!(BlockListener::attached_source(), None);
    // Nothing is attached again with no listener.
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(BlockGame::added().len(), 1);
}

#[test]
fn an_off_thread_change_is_raised_once_at_the_next_frame() {
    let harness = install(&[]);
    BlockGame::set_dimension(2);
    let listener = attached(&harness);

    std::thread::scope(|scope| {
        scope.spawn(|| BlockGame::fire_changed(listener, [5, 70, -9], 0, dirt(), stone())).join().unwrap();
    });
    assert_eq!(seen(), Vec::<String>::new());
    ClientGame::frame();
    ClientGame::frame();

    assert_eq!(seen(), ["dim=2 pos=5,70,-9 layer=0 new=minecraft:dirt old=minecraft:stone thread=0"]);
}

#[test]
fn world_left_takes_the_listener_out_and_drops_the_queued_changes() {
    let harness = install(&[]);
    let listener = attached(&harness);
    std::thread::scope(|scope| {
        scope.spawn(|| BlockGame::fire_changed(listener, [5, 70, -9], 0, dirt(), stone())).join().unwrap();
    });

    SessionGame::exit();
    ClientGame::frame();

    assert_eq!(BlockGame::removed(), [listener]);
    assert_eq!(BlockListener::attached_source(), None);
    assert_eq!(seen(), Vec::<String>::new());
}

#[test]
fn a_destroyed_source_is_forgotten_without_a_call_to_the_game() {
    let harness = install(&[]);
    let listener = attached(&harness);

    BlockGame::fire_destroyed(listener, ActorGame::block_source());
    ClientGame::update(&GAME_CLIENT_A);

    // No `removeListener` on a source that is going away; the listener is attached again to the source that is there.
    assert_eq!(BlockGame::removed(), Vec::<usize>::new());
    assert_eq!(BlockGame::added().len(), 2);
    assert_eq!(BlockListener::attached_source(), Some(ActorGame::block_source()));
}

#[test]
fn the_event_is_not_offered_without_the_add_listener_entry() {
    let harness = install(&["BlockSource.addListener"]);

    assert!(!harness.state.events().is_available("world.block_changed", 1));
    let logs = LogFile::wait_for(&harness.logs().join("loader.log"), "BlockSource.addListener");
    assert!(logs.contains("event not available"), "{logs}");
    assert!(logs.contains("BlockSource.addListener"), "{logs}");
}

#[test]
fn the_object_is_reused_at_every_attach_and_stays_readable_after_a_detach() {
    let harness = install(&[]);
    join(&harness);
    let id = listen(&harness);
    ClientGame::update(&GAME_CLIENT_A);
    let first = BlockGame::added()[0];
    let slots = BlockGame::slots_of(first);

    assert!(harness.state.events().unlisten(id));
    ClientGame::update(&GAME_CLIENT_A);

    assert_eq!(BlockGame::removed(), [first]);
    // The detached object is still readable, with the same table and the locator of the game's table.
    assert_eq!(BlockGame::slots_of(first), slots);
    assert_eq!(BlockGame::locator_of(first), BlockGame::locator());
    listen(&harness);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(BlockGame::added(), [first, first]);
}

#[test]
fn a_table_with_a_tenth_slot_is_refused() {
    let harness = Harness::start_with_entries(&BlockGame::entries(&[]));
    BlockGame::set_word_after_table(true);
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);

    let rows = BlockListener::install(harness.state, BlockGame::image);

    assert!(!harness.state.events().is_available("world.block_changed", 1));
    assert_eq!(rows[0].missing, vec!["BlockSourceListener slot count"]);
}

#[test]
fn a_table_that_ends_after_nine_slots_is_offered() {
    let harness = Harness::start_with_entries(&BlockGame::entries(&[]));
    BlockGame::set_word_after_table(false);
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);

    BlockListener::install(harness.state, BlockGame::image);

    assert!(harness.state.events().is_available("world.block_changed", 1));
}

#[test]
fn a_table_whose_end_lies_outside_the_image_is_refused() {
    let harness = Harness::start_with_entries(&BlockGame::entries(&[]));
    BlockGame::set_word_after_table(false);
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);

    let rows = BlockListener::install(harness.state, || ImageLayout::new(1, Vec::new()));

    assert!(!harness.state.events().is_available("world.block_changed", 1));
    assert_eq!(rows[0].missing, vec!["BlockSourceListener slot count"]);
}

const FIRST_MOD: &str = "@events/events";
const SECOND_MOD: &str = "@echo/echo";

// What the `new_state` listener saw: the token of the event, then the answers of `block_changed_state` for the first
// mod (twice), the second mod, and a token that is not the event's.
static STATES: Mutex<Vec<(AmmlStatus, AmmlBlockHandle)>> = Mutex::new(Vec::new());
static TOKEN: Mutex<u32> = Mutex::new(0);

fn world_table() -> AmmlWorldTable {
    let state = LoaderState::get().unwrap();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::world(services) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlWorldTable::read(table) }.unwrap()
}

fn changed_state(module: &str, token: u64) -> (AmmlStatus, AmmlBlockHandle) {
    let mut out = AmmlBlockHandle::NONE;
    // SAFETY: the table is live; the module text and the output live for the call.
    let status = unsafe { (world_table().block_changed_state.unwrap())(AmmlStr::from_str(module), token, &mut out) };
    (status, out)
}

unsafe extern "C" fn on_state(
    event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `world.block_changed` with a `BlockChanged`.
    let changed = unsafe { &*event.cast::<BlockChanged>() };
    let token = u64::from(changed.new_state);
    *TOKEN.lock().unwrap() = changed.new_state;
    let mut seen = STATES.lock().unwrap();
    seen.push(changed_state(FIRST_MOD, token));
    seen.push(changed_state(FIRST_MOD, token));
    seen.push(changed_state(SECOND_MOD, token));
    seen.push(changed_state(FIRST_MOD, token + 1));
    AmmlStatus::OK
}

// Installed over the `world.blocks` stand-ins, joined, with two mods loaded and the `new_state` listener attached.
fn attached_with_state() -> (Harness, usize) {
    STATES.lock().unwrap().clear();
    let harness = Harness::start_with_entries(&BlockReadGame::entries(&[]));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ActorAdapter::install(harness.state);
    WorldAdapter::install(harness.state);
    BlockListener::install(harness.state, BlockGame::image);
    BlockReadAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    harness.install_mod("echo", "echo", &EchoMod::build("plain", &[]), "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    join(&harness);
    let listener = ModListener { owner: 0, callback: on_state, user: 0 };
    harness.state.events().listen("world.block_changed", 2, AmmlBand::NORMAL, listener).unwrap();
    ClientGame::update(&GAME_CLIENT_A);
    let added = BlockGame::added();
    assert_eq!(added.len(), 1);
    (harness, added[0])
}

#[test]
fn new_state_interns_for_the_calling_mod() {
    let (_harness, listener) = attached_with_state();

    BlockGame::fire_changed(listener, [1, 2, 3], 0, stone(), dirt());

    let seen = STATES.lock().unwrap().clone();
    assert_eq!(seen.len(), 4);
    assert_eq!((seen[0].0, seen[1].0, seen[2].0), (AmmlStatus::OK, AmmlStatus::OK, AmmlStatus::OK));
    assert_ne!(seen[0].1, AmmlBlockHandle::NONE);
    assert_eq!(seen[0].1, seen[1].1);
    assert_ne!(seen[0].1, seen[2].1);
    assert_ne!(*TOKEN.lock().unwrap(), 0);
    assert_eq!(seen[3], (AmmlStatus::STALE, AmmlBlockHandle::NONE));
}

#[test]
fn new_state_outside_the_call_is_none() {
    let (_harness, listener) = attached_with_state();
    BlockGame::fire_changed(listener, [1, 2, 3], 0, stone(), dirt());
    let token = u64::from(*TOKEN.lock().unwrap());
    assert_ne!(token, 0);

    let after = changed_state(FIRST_MOD, token);

    assert_eq!(after, (AmmlStatus::STALE, AmmlBlockHandle::NONE));
}
