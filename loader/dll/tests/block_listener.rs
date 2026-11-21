use std::ffi::c_void;
use std::sync::Mutex;

use amml::{ActorAdapter, BlockListener, ClientAdapter, ModListener, SessionAdapter, WorldAdapter};
use amml_abi::{AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlStatus, BlockChanged};
use amml_runtime::{ImageLayout, ListenerId};

use crate::support::{
    ActorGame, BlockGame, ChunkGame, ChunkKind, ClientGame, GAME_CLIENT_A, Harness, LogFile, SessionGame,
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
