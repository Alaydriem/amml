use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use amml::{ClientAdapter, ModListener, RouterAdapter, SessionAdapter};
use amml_abi::{AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlRouteVia, AmmlStatus, IdleScreenOpen};
use amml_data::Availability;

use crate::support::{ClientGame, GAME_CLIENT_A, Harness, RouterGame};

// What the idle screen listener saw: `idle via=<via> client=<context client>`.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// Whether the listener cancels.
static CANCEL: AtomicBool = AtomicBool::new(true);

unsafe extern "C" fn on_idle(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `client.idle_screen_open` with an `IdleScreenOpen` and a live context.
    let (event, context) = unsafe { (&mut *event.cast::<IdleScreenOpen>(), &*context) };
    SEEN.lock().unwrap().push(format!("idle via={} client={:#x}", event.via.0, context.client as usize));
    if CANCEL.load(Ordering::SeqCst) {
        event.header.cancelled = 1;
    }
    event.via = AmmlRouteVia(99);
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

// Installs the session, client and router adapters over the stand-ins without `without`, after one frame and one
// update of client A (the primary client).
fn install(without: &[&str]) -> (Harness, Vec<Availability>) {
    let harness = Harness::start_with_entries(&RouterGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    let rows = RouterAdapter::install(harness.state);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    (harness, rows)
}

fn listen(harness: &Harness) {
    let listener = ModListener { owner: 0, callback: on_idle, user: 0 };
    assert!(harness.state.events().listen("client.idle_screen_open", 1, AmmlBand::NORMAL, listener).is_ok());
}

#[test]
fn a_cancelled_idle_push_is_refused_and_the_router_is_not_called() {
    let (harness, rows) = install(&[]);
    assert_eq!(rows[0].shape, Some("default"));
    listen(&harness);

    assert_eq!(RouterGame::push("/idle-route", 1), 0);

    assert!(RouterGame::calls().is_empty());
    let client = ClientGame::client(&GAME_CLIENT_A) as usize;
    assert_eq!(seen(), [format!("idle via=1 client={client:#x}")]);
}

#[test]
fn a_cancelled_idle_replace_with_a_prefix_is_refused() {
    let (harness, _) = install(&[]);
    listen(&harness);

    assert_eq!(RouterGame::replace("/screens/menu/idle-route"), 0);

    assert!(RouterGame::calls().is_empty());
    assert_eq!(seen().len(), 1);
    assert!(seen()[0].starts_with("idle via=2"));
}

#[test]
fn an_idle_route_no_listener_cancels_reaches_the_router() {
    let (harness, _) = install(&[]);
    CANCEL.store(false, Ordering::SeqCst);
    listen(&harness);

    assert_eq!(RouterGame::push("/idle-route", 3), 1);

    assert_eq!(RouterGame::calls(), ["push /idle-route mode=3"]);
}

#[test]
fn other_routes_raise_nothing_and_reach_the_router() {
    let (harness, _) = install(&[]);
    listen(&harness);

    assert_eq!(RouterGame::push("/hud", 0), 1);
    assert_eq!(RouterGame::replace("/idle-route/more"), 1);

    assert!(seen().is_empty());
    assert_eq!(RouterGame::calls(), ["push /hud mode=0", "replace /idle-route/more"]);
}

#[test]
fn without_a_listener_the_idle_route_reaches_the_router() {
    let _ = install(&[]);

    assert_eq!(RouterGame::push("/idle-route", 1), 1);

    assert_eq!(RouterGame::calls(), ["push /idle-route mode=1"]);
}

#[test]
fn a_missing_router_entry_leaves_the_event_not_offered() {
    let (harness, rows) = install(&[RouterGame::REPLACE_ROUTE]);

    assert_eq!(rows[0].shape, None);
    assert_eq!(rows[0].missing, vec![RouterGame::REPLACE_ROUTE]);
    let listener = ModListener { owner: 0, callback: on_idle, user: 0 };
    assert!(harness.state.events().listen("client.idle_screen_open", 1, AmmlBand::NORMAL, listener).is_err());
}
