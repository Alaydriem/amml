use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use amml::{ClientAdapter, ModListener, RouterAdapter, SessionActions, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlClientTable, AmmlEventContext, AmmlEventHeader, AmmlRouteVia, AmmlServices, AmmlStatus,
    IdleScreenOpen,
};
use amml_data::Availability;

use crate::support::{ClientGame, GAME_CLIENT_A, Harness, LogFile, RouterGame};

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
    SessionActions::install(harness.state);
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

fn table(harness: &Harness) -> &'static AmmlClientTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::client(services) }.unwrap();
    // SAFETY: as above.
    unsafe { &*table }
}

fn on_title(harness: &Harness) -> (AmmlStatus, u8) {
    let client = ClientGame::client(&GAME_CLIENT_A).cast_const();
    let mut out = 0xEE;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { table(harness).title_screen.unwrap()(client, &mut out) };
    (status, out)
}

// Runs `count` frames.
fn frames(count: u64) {
    for _ in 0..count {
        ClientGame::frame();
    }
}

#[test]
fn the_title_read_follows_the_last_route_the_router_took_once_it_stood_for_its_frames() {
    let (harness, rows) = install(&[]);
    assert!(rows.iter().any(|row| row.id == "client.title_screen" && row.shape.is_some()));
    assert!(harness.state.areas().is_available("client.title_screen", 1));
    let settle = RouterAdapter::TITLE_FRAMES;

    let before = on_title(&harness);
    RouterGame::push("/main-menu", 1);
    let just_taken = on_title(&harness);
    frames(settle);
    let title = on_title(&harness);
    RouterGame::push("/play/all?dirtyLevelId=", 1);
    frames(settle);
    let worlds = on_title(&harness);
    RouterGame::replace("/main-menu/?shouldFlushStack=true");
    frames(settle);
    let flushed = on_title(&harness);
    RouterGame::push("/main-menu-settings", 1);
    frames(settle);
    let other = on_title(&harness);

    assert_eq!(
        [before, just_taken, title, worlds, flushed, other],
        [
            (AmmlStatus::OK, 0),
            (AmmlStatus::OK, 0),
            (AmmlStatus::OK, 1),
            (AmmlStatus::OK, 0),
            (AmmlStatus::OK, 1),
            (AmmlStatus::OK, 0)
        ]
    );
}

// dejavu 70.26 r5, 17:31:14: the mod asked for the title and joined 1 ms later, before the title was shown.
#[test]
fn the_title_read_is_false_right_after_go_to_title_until_the_route_stood_for_its_frames() {
    let (harness, _) = install(&[]);
    let client = ClientGame::client(&GAME_CLIENT_A).cast_const();
    RouterGame::push("/play/all", 1);
    frames(RouterAdapter::TITLE_FRAMES);

    // SAFETY: the table is live.
    let asked = unsafe { table(&harness).go_to_title.unwrap()(client) };
    let at_once = on_title(&harness);
    frames(RouterAdapter::TITLE_FRAMES - 1);
    let one_frame_short = on_title(&harness);
    frames(1);
    let shown = on_title(&harness);

    assert_eq!(
        (asked, at_once, one_frame_short, shown),
        (AmmlStatus::OK, (AmmlStatus::OK, 0), (AmmlStatus::OK, 0), (AmmlStatus::OK, 1))
    );
}

// dejavu 70.26 r5, 17:30:22..17:30:28: the worlds page was shown after a leave, yet the read said true. A main menu
// route the router was asked for but did not take (the router gave false) must not count.
#[test]
fn a_main_menu_route_the_router_refuses_leaves_the_worlds_page_not_the_title() {
    let (harness, _) = install(&[]);
    RouterGame::push("/main-menu", 1);
    frames(RouterAdapter::TITLE_FRAMES);
    RouterGame::push("/play/all", 1);
    RouterGame::refuse("/main-menu");
    RouterGame::refuse("/main-menu/?shouldFlushStack=true");

    let pushed = RouterGame::push("/main-menu", 1);
    let replaced = RouterGame::replace("/main-menu/?shouldFlushStack=true");
    frames(RouterAdapter::TITLE_FRAMES);

    assert_eq!((pushed, replaced, on_title(&harness)), (0, 0, (AmmlStatus::OK, 0)));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "route not taken");
    assert!(log.contains("/main-menu"), "{log}");
}

#[test]
fn the_title_read_is_false_in_a_world_and_not_available_without_the_router_hooks() {
    let (harness, _) = install(&[]);
    RouterGame::push("/main-menu", 1);
    frames(RouterAdapter::TITLE_FRAMES);
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);

    assert_eq!(on_title(&harness), (AmmlStatus::OK, 0));
}

#[test]
fn the_title_area_is_not_offered_without_the_replace_hook() {
    let (harness, rows) = install(&[RouterGame::REPLACE_ROUTE]);

    let row = rows.iter().find(|row| row.id == "client.title_screen").unwrap();
    assert_eq!((row.shape, row.missing.clone()), (None, vec![RouterGame::REPLACE_ROUTE]));
    assert_eq!(on_title(&harness).0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn go_to_title_pushes_the_flushed_main_menu_route_and_is_refused_in_a_world() {
    let (harness, _) = install(&[]);
    assert!(harness.state.areas().is_available("client.go_to_title", 1));
    let client = ClientGame::client(&GAME_CLIENT_A).cast_const();
    let go = table(&harness).go_to_title.unwrap();

    // SAFETY: the table is live.
    let menu = unsafe { go(client) };
    frames(RouterAdapter::TITLE_FRAMES);
    let title = on_title(&harness);
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);
    // SAFETY: as above.
    let in_world = unsafe { go(client) };

    assert_eq!((menu, title, in_world), (AmmlStatus::OK, (AmmlStatus::OK, 1), AmmlStatus::REFUSED));
    assert_eq!(RouterGame::calls(), ["push /main-menu/?shouldFlushStack=true mode=0"]);
}
