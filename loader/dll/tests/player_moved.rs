use std::ffi::c_void;
use std::sync::Mutex;

use amml::{ClientAdapter, ModListener, PlayerMovedAdapter, SessionAdapter};
use amml_abi::{AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlPhase, AmmlStatus, AmmlThread, PlayerMoved};

use crate::support::{BodyGame, Harness, LogFile, SessionGame};

const OWNER: usize = 3;
const EVENT: &str = "player.moved";

// What the `PlayerMoved` listener saw, one line per call.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The client of each `PlayerMoved` context, one per call.
static CLIENTS: Mutex<Vec<usize>> = Mutex::new(Vec::new());

fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&BodyGame::entries_without(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    PlayerMovedAdapter::install(harness.state);
    harness
}

// The adapters over every stand-in entry, with a joined world whose local player is `BodyGame::LOCAL`.
fn start() -> (Harness, BodyGame) {
    let harness = install(&[]);
    let game = BodyGame::build();
    game.join();
    (harness, game)
}

unsafe extern "C" fn record(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live `PlayerMoved` and its context for the call.
    let (event, context) = unsafe { (&*event.cast::<PlayerMoved>(), &*context) };
    CLIENTS.lock().unwrap().push(context.client as usize);
    SEEN.lock().unwrap().push(format!(
        "pos={:?} pitch={} yaw={} head={} mode={} phase={} game_thread={}",
        [event.pos.x, event.pos.y, event.pos.z],
        event.rotation.pitch,
        event.rotation.yaw,
        event.head_yaw,
        event.mode,
        event.header.phase.0,
        context.thread == AmmlThread::GAME
    ));
    AmmlStatus::OK
}

fn listen(harness: &Harness) {
    let listener = ModListener { owner: OWNER, callback: record, user: 0 };
    harness.state.events().listen(EVENT, 1, AmmlBand::NORMAL, listener).unwrap();
}

fn seen() -> Vec<String> {
    std::mem::take(&mut *SEEN.lock().unwrap())
}

#[test]
fn the_event_is_offered_with_its_entries() {
    let harness = install(&[]);

    assert!(harness.state.events().is_available(EVENT, 1));
}

// Without `entry` the event is not offered.
fn withdrawn(entry: &str) {
    let harness = install(&[entry]);

    assert!(!harness.state.events().is_available(EVENT, 1), "{entry}");
}

#[test]
fn a_missing_handler_slot_withdraws_the_event() {
    withdrawn("ClientNetworkHandler.handle.MovePlayerPacket");
}

#[test]
fn a_missing_handler_table_withdraws_the_event() {
    withdrawn("LegacyClientNetworkHandler.vtable");
}

#[test]
fn a_missing_runtime_id_read_withdraws_the_event() {
    withdrawn("Actor.getRuntimeID");
}

#[test]
fn a_missing_local_player_read_withdraws_the_event() {
    withdrawn("ClientInstance.getLocalPlayer");
}

#[test]
fn player_moved_only_for_the_local_runtime_id() {
    let (harness, _game) = start();
    listen(&harness);
    let local = BodyGame::runtime_id(BodyGame::LOCAL);
    let other = BodyGame::runtime_id(BodyGame::OTHER);

    BodyGame::handle_move(BodyGame::move_packet(other, [1.0, 2.0, 3.0], [0.0, 0.0], 0.0, 0));
    assert!(seen().is_empty());

    BodyGame::handle_move(BodyGame::move_packet(local, [10.5, 64.0, -3.25], [15.0, 90.0], 45.0, 2));

    // The game's own handler ran for both packets; the event is raised After it, for the local player only.
    assert_eq!(BodyGame::moves_handled(), 2);
    assert_eq!(
        seen(),
        [format!(
            "pos={:?} pitch=15 yaw=90 head=45 mode=2 phase={} game_thread=true",
            [10.5f32, 64.0, -3.25],
            AmmlPhase::AFTER.0
        )]
    );
}

#[test]
fn rotation_follows_the_angle_rule() {
    let (harness, _game) = start();
    listen(&harness);
    let local = BodyGame::runtime_id(BodyGame::LOCAL);

    // The packet's rotation is pitch then yaw, in degrees: the order and signs of `actor.read`'s rotation.
    BodyGame::handle_move(BodyGame::move_packet(local, [0.0; 3], [-30.0, 135.0], -45.0, 0));

    let lines = seen();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("pitch=-30 yaw=135 head=-45"), "{}", lines[0]);
}

#[test]
fn without_a_listener_the_handler_runs_and_nothing_is_raised() {
    let (_harness, _game) = start();

    BodyGame::handle_move(BodyGame::move_packet(BodyGame::runtime_id(BodyGame::LOCAL), [0.0; 3], [0.0; 2], 0.0, 0));

    assert_eq!(BodyGame::moves_handled(), 1);
    assert!(seen().is_empty());
}

#[test]
fn no_event_before_a_world_is_joined() {
    let harness = install(&[]);
    let _game = BodyGame::build();
    listen(&harness);

    BodyGame::handle_move(BodyGame::move_packet(BodyGame::runtime_id(BodyGame::LOCAL), [0.0; 3], [0.0; 2], 0.0, 0));

    assert_eq!(BodyGame::moves_handled(), 1);
    assert!(seen().is_empty());
}

// dejavu 70.26 r8: the replay server sends the viewer's MovePlayer (Respawn) right after StartGame, before the loader's
// world join; the packet is kept and raised right after `WorldJoined`.
#[test]
fn a_local_player_packet_before_the_world_join_is_raised_after_the_join() {
    let harness = install(&[]);
    let game = BodyGame::build();
    listen(&harness);
    let local = BodyGame::runtime_id(BodyGame::LOCAL);

    game.before_join();
    BodyGame::handle_move(BodyGame::move_packet(local, [1.0, 70.0, 2.0], [10.0, 20.0], 20.0, 3));
    let before = seen();
    game.finish_join();

    assert!(before.is_empty(), "{before:?}");
    assert_eq!(
        seen(),
        [format!(
            "pos={:?} pitch=10 yaw=20 head=20 mode=3 phase={} game_thread=true",
            [1.0f32, 70.0, 2.0],
            AmmlPhase::AFTER.0
        )]
    );
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "player moved raised");
    assert!(log.contains("player moved kept until the world join"), "{log}");
}

#[test]
fn a_kept_packet_is_dropped_when_another_join_starts() {
    let harness = install(&[]);
    let game = BodyGame::build();
    listen(&harness);

    game.before_join();
    BodyGame::handle_move(BodyGame::move_packet(BodyGame::runtime_id(BodyGame::LOCAL), [0.0; 3], [0.0; 2], 0.0, 3));
    BodyGame::start_join_again();
    game.finish_join();

    assert!(seen().is_empty());
}

// A packet kept for a join the player left before the world join is never raised into a later world.
#[test]
fn a_kept_packet_is_dropped_when_the_player_leaves_before_the_world_join() {
    let harness = install(&[]);
    let game = BodyGame::build();
    listen(&harness);

    game.before_join();
    BodyGame::handle_move(BodyGame::move_packet(BodyGame::runtime_id(BodyGame::LOCAL), [5.0; 3], [0.0; 2], 0.0, 3));
    SessionGame::exit();
    game.finish_join();

    assert!(seen().is_empty());
}

#[test]
fn only_the_latest_kept_packet_is_raised_after_the_join() {
    let harness = install(&[]);
    let game = BodyGame::build();
    listen(&harness);
    let local = BodyGame::runtime_id(BodyGame::LOCAL);

    game.before_join();
    BodyGame::handle_move(BodyGame::move_packet(local, [1.0; 3], [0.0; 2], 0.0, 1));
    BodyGame::handle_move(BodyGame::move_packet(local, [2.0, 3.0, 4.0], [0.0; 2], 0.0, 3));
    game.finish_join();

    let lines = seen();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains(&format!("pos={:?}", [2.0f32, 3.0, 4.0])) && lines[0].contains("mode=3"), "{}", lines[0]);
}

#[test]
fn each_skip_reason_is_logged_once() {
    let (harness, _game) = start();
    listen(&harness);
    let other = BodyGame::runtime_id(BodyGame::OTHER);

    BodyGame::handle_move(BodyGame::move_packet(other, [0.0; 3], [0.0; 2], 0.0, 0));
    BodyGame::handle_move(BodyGame::move_packet(other, [0.0; 3], [0.0; 2], 0.0, 0));

    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "player moved skipped");
    assert_eq!(log.matches("player moved skipped").count(), 1, "{log}");
    assert!(log.contains("another actor") && log.contains(&format!("{other}")), "{log}");
}

// dejavu 70.26 r9: the listener reads the local player through `EventCtx::client()`; the context carries the primary
// client, for a raise in a joined world and for the packet kept until the world join.
#[test]
fn the_context_carries_the_primary_client() {
    let harness = install(&[]);
    let game = BodyGame::build();
    listen(&harness);
    let local = BodyGame::runtime_id(BodyGame::LOCAL);

    game.before_join();
    BodyGame::handle_move(BodyGame::move_packet(local, [0.0; 3], [0.0; 2], 0.0, 1));
    game.finish_join();
    BodyGame::handle_move(BodyGame::move_packet(local, [1.0; 3], [0.0; 2], 0.0, 0));

    assert_eq!(seen().len(), 2);
    assert_eq!(*CLIENTS.lock().unwrap(), [game.client(), game.client()]);
}
