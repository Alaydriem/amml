use std::ffi::c_void;
use std::sync::Mutex;

use amml::{ClientAdapter, ModListener, PlayerActionAdapter, SessionAdapter, UseBuildDetours};
use amml_abi::{
    AmmlBand, AmmlBlockPos, AmmlEventContext, AmmlEventHeader, AmmlStatus, AmmlThread, BuildBlock, UsePressed,
};

use crate::support::{BuildGame, ClientGame, GAME_CLIENT_A, Harness, LogFile};

const USE: &str = "player.use_pressed";
const BUILD: &str = "player.build_block";

// What the listeners saw, in order.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

// Installs the session, client and player action adapters over every stand-in entry but `without`.
fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&BuildGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    let rows = PlayerActionAdapter::install(harness.state);
    for event in [USE, BUILD] {
        assert!(rows.iter().any(|row| row.id == event && row.shape.is_some()), "{rows:?}");
    }
    harness
}

// `install` with every entry, after one frame (the gate is open, this thread is the game thread) and a joined world
// whose local player is the stand-in player.
fn start() -> Harness {
    let harness = install(&[]);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(BuildGame::player());
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

// Listens to `event` as owner 9; `user` 1 cancels.
fn listen(harness: &Harness, event: &str, cancel: bool) -> amml_runtime::ListenerId {
    let callback = if event == USE { on_use } else { on_build };
    let listener = ModListener { owner: 9, callback, user: usize::from(cancel) };
    harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap()
}

unsafe extern "C" fn on_use(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `player.use_pressed` with a `UsePressed` and a live context.
    let (event, context) = unsafe { (&mut *event.cast::<UsePressed>(), &*context) };
    SEEN.lock().unwrap().push(format!(
        "use thread={} client={}",
        context.thread.0,
        usize::from(!context.client.is_null())
    ));
    if !user.is_null() {
        event.header.cancelled = 1;
    }
    AmmlStatus::OK
}

unsafe extern "C" fn on_build(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `player.build_block` with a `BuildBlock` and a live context.
    let (event, context) = unsafe { (&mut *event.cast::<BuildBlock>(), &*context) };
    SEEN.lock().unwrap().push(format!(
        "build thread={} client={} local={} pos={},{},{} face={} hand={} sim={} origin={}",
        context.thread.0,
        usize::from(!context.client.is_null()),
        event.is_local_player,
        event.pos.x,
        event.pos.y,
        event.pos.z,
        event.face.0,
        event.hand.0,
        event.sim_tick,
        event.header.origin.0
    ));
    if !user.is_null() {
        event.header.cancelled = 1;
    }
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

const POS: AmmlBlockPos = AmmlBlockPos { x: 4, y: 70, z: -2 };

#[test]
fn cancelled_use_returns_handled() {
    let harness = start();
    listen(&harness, USE, true);

    let result = BuildGame::game_use();

    assert_eq!(result, UseBuildDetours::HANDLED);
    assert!(BuildGame::calls().is_empty(), "{:?}", BuildGame::calls());
    assert_eq!(seen(), [format!("use thread={} client=1", AmmlThread::GAME.0)]);
}

#[test]
fn a_use_no_listener_cancels_reaches_the_game_with_its_arguments() {
    let harness = start();
    listen(&harness, USE, false);

    let result = BuildGame::game_use();

    assert_eq!(result, BuildGame::GAME_USE_RESULT);
    assert_eq!(BuildGame::calls(), ["use client=0x100 intention=0x200 solid=0x300 liquid=0x400"]);
    assert_eq!(seen().len(), 1);
}

#[test]
fn a_game_thread_build_of_the_local_player_carries_the_client_and_the_fields() {
    let harness = start();
    listen(&harness, BUILD, false);

    let result = BuildGame::game_build(BuildGame::game_mode(), POS, 0xAB01, 0xCD00, 0xEF01);

    assert_eq!(result, 1);
    assert_eq!(
        seen(),
        [format!("build thread={} client=1 local=1 pos=4,70,-2 face=1 hand=0 sim=1 origin=0", AmmlThread::GAME.0)]
    );
    assert_eq!(BuildGame::calls().len(), 1);
    assert!(BuildGame::calls()[0].contains("face=0xab01 hand=0xcd00 sim=0xef01"), "{:?}", BuildGame::calls());
}

#[test]
fn a_build_of_another_player_is_not_local() {
    let harness = start();
    listen(&harness, BUILD, false);
    BuildGame::set_game_mode_player(BuildGame::player() + 8);

    BuildGame::game_build(BuildGame::game_mode(), POS, 1, 0, 0);

    assert!(seen()[0].contains("local=0"), "{:?}", seen());
}

#[test]
fn server_thread_build_has_no_client() {
    let harness = start();
    listen(&harness, BUILD, false);

    let result = std::thread::scope(|scope| {
        scope.spawn(|| BuildGame::game_build(BuildGame::game_mode(), POS, 2, 0, 1)).join().unwrap()
    });
    BuildGame::game_build(BuildGame::game_mode(), POS, 2, 0, 1);

    assert_eq!(result, 1);
    assert_eq!(
        seen(),
        [
            format!("build thread={} client=0 local=0 pos=4,70,-2 face=2 hand=0 sim=1 origin=0", AmmlThread::SERVER.0),
            format!("build thread={} client=1 local=1 pos=4,70,-2 face=2 hand=0 sim=1 origin=0", AmmlThread::GAME.0),
        ]
    );
    assert_eq!(BuildGame::calls().len(), 2);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "\"game\":true");
    assert_eq!(log.matches("build block thread").count(), 2, "{log}");
}

#[test]
fn a_cancelled_build_returns_false_without_the_game() {
    let harness = start();
    listen(&harness, BUILD, true);

    let result = BuildGame::game_build(BuildGame::game_mode(), POS, 1, 0, 0);

    assert_eq!(result, 0);
    assert!(BuildGame::calls().is_empty(), "{:?}", BuildGame::calls());
}

#[test]
fn no_listener_reads_one_flags_word() {
    let harness = start();
    assert_eq!(UseBuildDetours::flags(), 0);
    // A GameMode in the first page, which is never mapped: a read of its `mPlayer` would end the test process.
    let unmapped = 0x10 as *mut c_void;

    let result = BuildGame::game_build(unmapped, POS, 1, 0, 1);

    assert_eq!(result, 1);
    assert_eq!(BuildGame::calls().len(), 1);
    assert!(BuildGame::calls()[0].starts_with("build this=0x10 pos=4,70,-2 face=0x1 hand=0x0 sim=0x1"));
    assert_eq!(UseBuildDetours::last_sim_tick(harness.state.events().gate().epoch().0), 0);
    let use_listener = listen(&harness, USE, false);
    let build_listener = listen(&harness, BUILD, false);
    assert_eq!(UseBuildDetours::flags(), UseBuildDetours::USE_LISTENER | UseBuildDetours::BUILD_LISTENER);
    assert!(harness.state.events().unlisten(use_listener));
    assert!(harness.state.events().unlisten(build_listener));
    assert_eq!(UseBuildDetours::flags(), 0);
    assert!(seen().is_empty());
}

#[test]
fn a_build_before_the_first_frame_reaches_the_game_with_no_event() {
    let harness = install(&[]);
    listen(&harness, BUILD, true);

    let result = BuildGame::game_build(BuildGame::game_mode(), POS, 1, 0, 0);

    assert_eq!(result, 1);
    assert_eq!(BuildGame::calls().len(), 1);
    assert!(seen().is_empty());
}

#[test]
fn the_events_are_not_offered_without_their_entries() {
    let harness = Harness::start_with_entries(&BuildGame::entries(&[
        "ClientInputCallbacks.handleBuildAction",
        "GameMode.buildBlock",
    ]));
    SessionAdapter::install(harness.state);
    let rows = PlayerActionAdapter::install(harness.state);

    for event in [USE, BUILD] {
        let row = rows.iter().find(|row| row.id == event).unwrap();
        assert_eq!(row.shape, None);
        assert!(!harness.state.events().is_available(event, 1));
    }
}
