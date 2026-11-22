use amml::{
    ActorRenderQueueLayout, BodyHideAdapter, BodyHideDetours, CameraAdapter, ClientAdapter, ModLoader, SessionAdapter,
};
use amml_abi::{AmmlActorTarget, AmmlApi, AmmlCameraTable, AmmlServices, AmmlStatus, AmmlStr};
use amml_runtime::ModRegistry;

use crate::support::{BodyGame, CLIENT_A, EventsMod, Harness, SessionGame};

// The module of the mod that `start` loads, and its index.
const MODULE: &str = "@events/events";
const OWNER: usize = 0;
const OTHER: usize = 1;
const AREA: &str = "camera.hide_actor";
const A: usize = 0x7000;
const B: usize = 0x7100;
const C: usize = 0x7200;

fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&BodyGame::entries_without(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    CameraAdapter::install(harness.state);
    BodyHideAdapter::install(harness.state);
    harness
}

// The adapters over every stand-in entry, with a joined world whose local player is `BodyGame::LOCAL`.
fn start() -> (Harness, BodyGame) {
    let harness = install(&[]);
    let game = BodyGame::build();
    game.join();
    (harness, game)
}

fn adapter() -> &'static BodyHideAdapter {
    BodyHideAdapter::get().unwrap()
}

fn hide(owner: usize, on: bool) -> Result<(), AmmlStatus> {
    adapter().hide_actor(owner, AmmlActorTarget::LOCAL_PLAYER, on)
}

// The camera table, as a mod reaches it, with the plain mod loaded at index 0.
fn table(harness: &Harness) -> AmmlCameraTable {
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::camera(services) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlCameraTable::read(table) }.unwrap()
}

#[test]
fn the_area_is_offered_with_its_entries() {
    let harness = install(&[]);

    assert!(harness.state.areas().is_available(AREA, 1));
}

// Without `entry` the area is not offered and a call says so.
fn withdrawn(entry: &str) {
    let harness = install(&[entry]);

    assert!(!harness.state.areas().is_available(AREA, 1), "{entry}");
    assert_eq!(hide(OWNER, true), Err(AmmlStatus::NOT_AVAILABLE), "{entry}");
}

#[test]
fn a_missing_queue_slot_withdraws_the_area() {
    withdrawn("LevelRendererPlayer.queueRenderEntities");
}

#[test]
fn a_missing_queue_member_withdraws_the_area() {
    withdrawn("LevelRendererCamera.mActorRenderQueue");
}

#[test]
fn a_missing_dispatcher_withdraws_the_area() {
    withdrawn("ActorRenderDispatcher.render");
}

#[test]
fn a_missing_local_player_read_withdraws_the_area() {
    withdrawn("ClientInstance.getLocalPlayer");
}

#[test]
fn queue_erase_moves_keys_and_values() {
    let (_harness, game) = start();
    hide(OWNER, true).unwrap();
    game.frame();
    let renderer = BodyGame::renderer(&[A, BodyGame::LOCAL, B, C], &[1, 2, 3, 4]);
    let [keys_begin, keys_end, keys_cap, values_begin, values_end, values_cap] = BodyGame::queue_pointers(renderer);

    BodyGame::queue_entities(renderer);

    // The original ran, then the pair of the local player left: the tails moved down, both ends went back by one
    // element, and no begin or capacity moved.
    assert_eq!(BodyGame::queued(), 1);
    assert_eq!(BodyGame::queue(renderer), [(A, 1), (B, 3), (C, 4)]);
    assert_eq!(
        BodyGame::queue_pointers(renderer),
        [keys_begin, keys_end - 8, keys_cap, values_begin, values_end - 1, values_cap]
    );

    // A second pass finds no local player: nothing changes.
    let after = BodyGame::queue_pointers(renderer);
    BodyGame::queue_entities(renderer);
    assert_eq!(BodyGame::queue_pointers(renderer), after);
    assert_eq!(BodyGame::queue(renderer), [(A, 1), (B, 3), (C, 4)]);
}

#[test]
fn queue_erase_takes_the_last_pair_and_a_lone_pair() {
    let (_harness, game) = start();
    hide(OWNER, true).unwrap();
    game.frame();
    let last = BodyGame::renderer(&[A, BodyGame::LOCAL], &[1, 2]);
    let lone = BodyGame::renderer(&[BodyGame::LOCAL], &[9]);

    BodyGame::queue_entities(last);
    BodyGame::queue_entities(lone);

    assert_eq!(BodyGame::queue(last), [(A, 1)]);
    assert!(BodyGame::queue(lone).is_empty());
}

#[test]
fn a_queue_that_is_not_the_games_shape_is_left_alone() {
    let (_harness, _game) = start();
    // Keys and values of different lengths, and a null queue: the game's own state stands.
    let uneven = BodyGame::renderer(&[A, BodyGame::LOCAL, B], &[1, 2]);
    let before = BodyGame::queue_pointers(uneven);
    // SAFETY: the stand-in renderer's queue is live and nothing else uses it.
    let erased = unsafe { ActorRenderQueueLayout::erase_actor(uneven + BodyGame::QUEUE_AT, BodyGame::LOCAL) };
    assert!(!erased);
    assert_eq!(BodyGame::queue_pointers(uneven), before);

    let empty = BodyGame::renderer(&[], &[]);
    // SAFETY: as above.
    assert!(!unsafe { ActorRenderQueueLayout::erase_actor(empty + BodyGame::QUEUE_AT, BodyGame::LOCAL) });
    // SAFETY: a zeroed queue has null begin pointers, which the erase refuses before it reads.
    assert!(!unsafe { ActorRenderQueueLayout::erase_actor(BodyGame::renderer(&[], &[]) + 0x10, 0) });
}

#[test]
fn dispatch_skip_matches_only_the_local_player() {
    let (_harness, game) = start();
    hide(OWNER, true).unwrap();
    game.frame();

    BodyGame::render(BodyGame::LOCAL);
    BodyGame::render(BodyGame::OTHER);

    // Only the other actor reached the game's draw, with its stack words as they came.
    assert_eq!(BodyGame::drawn(), [(BodyGame::OTHER, [1, 2, 3])]);

    hide(OWNER, false).unwrap();
    BodyGame::render(BodyGame::LOCAL);
    assert_eq!(BodyGame::drawn().last(), Some(&(BodyGame::LOCAL, [1, 2, 3])));
}

#[test]
fn nothing_hidden_reads_one_flag_and_calls_the_original() {
    let (_harness, _game) = start();
    assert!(!BodyHideDetours::hidden());
    let renderer = BodyGame::renderer(&[A, BodyGame::LOCAL], &[1, 2]);

    BodyGame::queue_entities(renderer);
    BodyGame::render(BodyGame::LOCAL);

    assert_eq!(BodyGame::queued(), 1);
    assert_eq!(BodyGame::queue(renderer), [(A, 1), (BodyGame::LOCAL, 2)]);
    assert_eq!(BodyGame::drawn(), [(BodyGame::LOCAL, [1, 2, 3])]);
}

#[test]
fn any_wish_keeps_the_body_hidden() {
    let (_harness, game) = start();
    hide(OWNER, true).unwrap();
    hide(OTHER, true).unwrap();
    game.frame();

    hide(OWNER, false).unwrap();
    BodyGame::render(BodyGame::LOCAL);
    assert!(BodyGame::drawn().is_empty());

    hide(OTHER, false).unwrap();
    BodyGame::render(BodyGame::LOCAL);
    assert_eq!(BodyGame::drawn().len(), 1);
    assert!(!BodyHideDetours::hidden());
}

#[test]
fn world_left_clears_the_hide() {
    let (_harness, game) = start();
    hide(OWNER, true).unwrap();
    game.frame();
    assert!(BodyHideDetours::hidden());

    SessionGame::exit();

    // Both paths show the body again: the queue keeps the local player and the dispatcher draws it.
    assert!(!BodyHideDetours::hidden());
    let renderer = BodyGame::renderer(&[BodyGame::LOCAL], &[1]);
    BodyGame::queue_entities(renderer);
    BodyGame::render(BodyGame::LOCAL);
    assert_eq!(BodyGame::queue(renderer), [(BodyGame::LOCAL, 1)]);
    assert_eq!(BodyGame::drawn().len(), 1);
}

#[test]
fn disable_clears_only_the_wish_of_that_mod() {
    let (_harness, game) = start();
    hide(OWNER, true).unwrap();
    hide(OTHER, true).unwrap();
    game.frame();

    adapter().disable_owner(OWNER);
    assert!(BodyHideDetours::hidden());
    adapter().disable_owner(OTHER);

    assert!(!BodyHideDetours::hidden());
    BodyGame::render(BodyGame::LOCAL);
    assert_eq!(BodyGame::drawn().len(), 1);
}

#[test]
fn a_new_world_takes_its_own_local_player() {
    let (_harness, game) = start();
    hide(OWNER, true).unwrap();
    game.frame();
    SessionGame::exit();
    hide(OWNER, true).unwrap();

    // Another player after the next join: the old one is drawn, the new one is hidden.
    SessionGame::set_player(BodyGame::OTHER);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update_raw(std::ptr::with_exposed_provenance_mut(game.client), 1, 0);
    game.frame();

    BodyGame::render(BodyGame::LOCAL);
    BodyGame::render(BodyGame::OTHER);
    assert_eq!(BodyGame::drawn(), [(BodyGame::LOCAL, [1, 2, 3])]);
}

#[test]
fn hide_actor_refuses_an_unknown_target() {
    let (_harness, _game) = start();

    assert_eq!(adapter().hide_actor(OWNER, AmmlActorTarget(0), true), Err(AmmlStatus::INVALID_ARGUMENT));
    assert_eq!(adapter().hide_actor(OWNER, AmmlActorTarget(9), true), Err(AmmlStatus::INVALID_ARGUMENT));
    assert!(!BodyHideDetours::hidden());
}

#[test]
fn hide_actor_works_from_any_thread() {
    let (_harness, game) = start();

    std::thread::spawn(|| hide(OWNER, true)).join().unwrap().unwrap();
    game.frame();

    BodyGame::render(BodyGame::LOCAL);
    assert!(BodyGame::drawn().is_empty());
}

#[test]
fn the_table_sets_the_wish_of_the_calling_mod() {
    let (harness, game) = start();
    let table = table(&harness);
    let hide_actor = table.hide_actor.unwrap();

    // SAFETY: the table is live and the module string lives for the call.
    let status = unsafe { hide_actor(AmmlStr::from_str(MODULE), AmmlActorTarget::LOCAL_PLAYER, 1) };
    assert_eq!(status, AmmlStatus::OK);
    game.frame();
    BodyGame::render(BodyGame::LOCAL);
    assert!(BodyGame::drawn().is_empty());

    // An unknown module and an unknown target are refused; the wish of the real mod stands.
    // SAFETY: as above.
    let unknown = unsafe { hide_actor(AmmlStr::from_str("@nobody/none"), AmmlActorTarget::LOCAL_PLAYER, 1) };
    assert_ne!(unknown, AmmlStatus::OK);
    // SAFETY: as above.
    let target = unsafe { hide_actor(AmmlStr::from_str(MODULE), AmmlActorTarget(7), 1) };
    assert_eq!(target, AmmlStatus::INVALID_ARGUMENT);

    // SAFETY: as above.
    assert_eq!(unsafe { hide_actor(AmmlStr::from_str(MODULE), AmmlActorTarget::LOCAL_PLAYER, 0) }, AmmlStatus::OK);
    BodyGame::render(BodyGame::LOCAL);
    assert_eq!(BodyGame::drawn().len(), 1);
}
