use std::ffi::c_void;
use std::sync::Mutex;

use amml::{ClientAdapter, FocusAdapter, FocusGainedDetours, ModListener, ModLoader, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlClientTable, AmmlEventContext, AmmlEventHeader, AmmlPhase, AmmlServices, AmmlStatus,
    AmmlStr, AmmlThread,
};
use amml_runtime::ModRegistry;

use crate::support::{ClientGame, EventsMod, FakeGameClient, GAME_CLIENT_A, GAME_CLIENT_B, Harness};

const LOST: &str = "client.app_focus_lost";
const GAINED: &str = "client.app_focus_gained";

// What the listeners saw: `<event> phase=<n> thread=<n> client=<0|1>`.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

// Installs the session, client and focus adapters over every stand-in entry but `without`.
fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&ClientGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    FocusAdapter::install(harness.state);
    harness
}

// `install` with every entry, after one frame (the gate is open, this thread is the game thread) and one update of
// client A, which makes it the primary client.
fn start() -> Harness {
    let harness = install(&[]);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

fn table(harness: &Harness) -> &'static AmmlClientTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::client(services) }.unwrap();
    // SAFETY: as above.
    unsafe { &*table }
}

fn handle(client: &'static FakeGameClient) -> *const c_void {
    ClientGame::client(client).cast_const()
}

fn replay(table: &AmmlClientTable, module: &str, client: *const c_void) -> AmmlStatus {
    // SAFETY: the table is live and the view lives for the call.
    unsafe { table.replay_app_focus_lost.unwrap()(AmmlStr::from_str(module), client) }
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

// Listens to `event` as the mod at `owner`; with `cancel`, the listener cancels the event.
fn listen(harness: &Harness, event: &str, owner: usize, cancel: bool) -> amml_runtime::ListenerId {
    let listener = ModListener { owner, callback: record, user: usize::from(cancel) };
    harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap()
}

unsafe extern "C" fn record(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live header and context for the call.
    let (header, context) = unsafe { (&mut *event, &*context) };
    SEEN.lock().unwrap().push(format!(
        "phase={} thread={} client={}",
        header.phase.0,
        context.thread.0,
        usize::from(!context.client.is_null())
    ));
    if !user.is_null() {
        header.cancelled = 1;
    }
    AmmlStatus::OK
}

#[test]
fn cancelled_loss_skips_the_game_handler() {
    let harness = start();
    listen(&harness, LOST, 9, true);

    ClientGame::game_focus_lost();

    assert!(ClientGame::focus_lost_this().is_empty());
    assert_eq!(
        *SEEN.lock().unwrap(),
        [format!("phase={} thread={} client=1", AmmlPhase::BEFORE.0, AmmlThread::GAME.0)]
    );
}

#[test]
fn a_loss_no_listener_cancels_reaches_the_game_once() {
    let harness = start();
    listen(&harness, LOST, 9, false);

    ClientGame::game_focus_lost();

    assert_eq!(ClientGame::focus_lost_this(), [ClientGame::game() + ClientGame::LISTENER_OFFSET]);
    assert_eq!(SEEN.lock().unwrap().len(), 1);
}

#[test]
fn replay_without_a_held_loss_runs_nothing() {
    let harness = start();
    enable_mods(&harness);
    // Mod 1 listens and does not cancel; mod 0 holds nothing.
    listen(&harness, LOST, 1, false);
    ClientGame::game_focus_lost();
    assert_eq!(ClientGame::focus_lost_this().len(), 1);

    let status = replay(table(&harness), "@events/events", handle(&GAME_CLIENT_A));

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(ClientGame::focus_lost_this().len(), 1);
}

#[test]
fn replay_runs_the_handler_once() {
    let harness = start();
    enable_mods(&harness);
    listen(&harness, LOST, 0, true);
    ClientGame::game_focus_lost();
    assert!(ClientGame::focus_lost_this().is_empty());
    let table = table(&harness);

    assert_eq!(replay(table, "@events/events", handle(&GAME_CLIENT_A)), AmmlStatus::OK);
    assert_eq!(replay(table, "@events/events", handle(&GAME_CLIENT_A)), AmmlStatus::OK);

    // The replay passed the detour: the handler ran once with the listener subobject and no event was raised again.
    assert_eq!(ClientGame::focus_lost_this(), [ClientGame::game() + ClientGame::LISTENER_OFFSET]);
    assert_eq!(SEEN.lock().unwrap().len(), 1);
}

#[test]
fn a_cancel_of_one_mod_is_not_the_replay_of_another() {
    let harness = start();
    enable_mods(&harness);
    listen(&harness, LOST, 0, true);
    ClientGame::game_focus_lost();

    assert_eq!(replay(table(&harness), "@events/events-provider", handle(&GAME_CLIENT_A)), AmmlStatus::OK);

    assert!(ClientGame::focus_lost_this().is_empty());
}

#[test]
fn a_gain_between_the_cancel_and_the_replay_forgets_the_loss() {
    let harness = start();
    enable_mods(&harness);
    listen(&harness, LOST, 0, true);
    ClientGame::game_focus_lost();

    ClientGame::game_focus_gained();
    let status = replay(table(&harness), "@events/events", handle(&GAME_CLIENT_A));

    assert_eq!(status, AmmlStatus::OK);
    assert!(ClientGame::focus_lost_this().is_empty());
    assert_eq!(ClientGame::focus_gains(), 1);
}

#[test]
fn a_gain_reaches_the_game_then_its_listeners() {
    let harness = start();
    listen(&harness, GAINED, 9, true);

    ClientGame::game_focus_gained();

    assert_eq!(ClientGame::focus_gains(), 1);
    // A cancel of a notification is ignored.
    assert_eq!(*SEEN.lock().unwrap(), [format!("phase={} thread={} client=1", AmmlPhase::AFTER.0, AmmlThread::GAME.0)]);
}

#[test]
fn replay_off_the_game_thread_is_refused() {
    let harness = start();
    enable_mods(&harness);
    listen(&harness, LOST, 0, true);
    ClientGame::game_focus_lost();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A) as usize;

    let status = std::thread::spawn(move || replay(table, "@events/events", client as *const c_void)).join().unwrap();

    assert_eq!(status, AmmlStatus::REFUSED);
    assert!(ClientGame::focus_lost_this().is_empty());
    // The refusal did not take the loss: a replay on the game thread still runs the handler.
    assert_eq!(replay(table, "@events/events", handle(&GAME_CLIENT_A)), AmmlStatus::OK);
    assert_eq!(ClientGame::focus_lost_this().len(), 1);
}

#[test]
fn replay_for_another_client_is_refused() {
    let harness = start();
    enable_mods(&harness);
    listen(&harness, LOST, 0, true);
    ClientGame::game_focus_lost();

    assert_eq!(replay(table(&harness), "@events/events", handle(&GAME_CLIENT_B)), AmmlStatus::REFUSED);

    assert!(ClientGame::focus_lost_this().is_empty());
}

#[test]
fn no_listener_reads_one_flags_word() {
    let harness = start();
    assert_eq!(FocusGainedDetours::flags(), 0);

    // A listener that came and went leaves the flags word clear again, so the detours raise nothing.
    let id = listen(&harness, LOST, 9, true);
    assert_eq!(FocusGainedDetours::flags(), FocusGainedDetours::LOSS_LISTENER);
    assert!(harness.state.events().unlisten(id));
    assert_eq!(FocusGainedDetours::flags(), 0);
    ClientGame::game_focus_lost();
    ClientGame::game_focus_gained();

    assert_eq!(ClientGame::focus_lost_this().len(), 1);
    assert_eq!(ClientGame::focus_gains(), 1);
    assert!(SEEN.lock().unwrap().is_empty());
}

#[test]
fn the_rows_are_not_offered_without_their_entries() {
    let harness = install(&["MinecraftGame.onAppFocusGained", "MinecraftGame.onAppFocusLost"]);

    let events = harness.state.events();
    assert!(!events.is_available(LOST, 1));
    assert!(!events.is_available(GAINED, 1));
    assert!(!harness.state.areas().is_available(FocusAdapter::REPLAY, 1));
}
