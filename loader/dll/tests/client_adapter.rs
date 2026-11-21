use std::ffi::c_void;
use std::sync::Mutex;

use amml::{ClientAdapter, ModListener, ModLoader, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlClientFlag, AmmlClientTable, AmmlEventContext, AmmlEventHeader, AmmlInputBlocks,
    AmmlLeaseMode, AmmlReleaseReason, AmmlScreen, AmmlServices, AmmlStatus, AmmlStr,
};
use amml_runtime::{LeaseTerms, ModRegistry};

use crate::support::{
    ActivityGame, ClientGame, EventsMod, FakeGameClient, GAME_CLIENT_A, GAME_CLIENT_B, Harness, HeapProbe, LogFile,
    RecordingFeeds, SessionGame,
};

const W: u16 = 0x57;

// What the `client.mouse_grab` listener saw: `grab client=<context client>`.
static GRABS_SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

// Installs the session and client adapters over every stand-in entry but `without`.
fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&ClientGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    harness
}

// `install` with every entry, after one frame (the gate is open, this thread is the game thread) and one
// update of client A, which makes it the primary client. No world is joined.
fn start() -> Harness {
    let harness = install(&[]);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

// `start`, then a joined world (epoch 1).
fn start_in_world() -> Harness {
    let harness = start();
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 1);
    harness
}

// The client table, through the services table as a mod reaches it.
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

fn flag(table: &AmmlClientTable, client: *const c_void, flag: AmmlClientFlag) -> (AmmlStatus, u8) {
    let mut out = 0xEEu8;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.flag)(client, flag, &mut out) };
    (status, out)
}

fn top_screen(table: &AmmlClientTable, client: *const c_void) -> (AmmlStatus, AmmlScreen) {
    let mut out = AmmlScreen(0xEE);
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.top_screen)(client, &mut out) };
    (status, out)
}

fn local_player(table: &AmmlClientTable, client: *const c_void) -> (AmmlStatus, usize) {
    let mut out: *const c_void = std::ptr::without_provenance(0xEE);
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.local_player)(client, &mut out) };
    (status, out as usize)
}

fn set_flag(
    table: &AmmlClientTable,
    module: &str,
    client: *const c_void,
    flag: AmmlClientFlag,
    value: u8,
) -> AmmlStatus {
    // SAFETY: the table is live and the view lives for the call.
    unsafe { (table.set_flag)(AmmlStr::from_str(module), client, flag, value) }
}

// Listens to `client.mouse_grab`; with `cancel`, the listener cancels the grab.
fn listen_grab(harness: &Harness, cancel: bool) {
    let listener = ModListener { owner: 9, callback: record_grab, user: usize::from(cancel) };
    harness.state.events().listen("client.mouse_grab", 1, AmmlBand::NORMAL, listener).unwrap();
}

unsafe extern "C" fn record_grab(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live header and context for the call.
    let (header, context) = unsafe { (&mut *event, &*context) };
    GRABS_SEEN.lock().unwrap().push(format!("grab client={:#x}", context.client as usize));
    if !user.is_null() {
        header.cancelled = 1;
    }
    AmmlStatus::OK
}

// Loads and enables the events-mod fixture variants `plain` (index 0) and, with `two`, `provider` (index 1).
fn enable_mods(harness: &Harness, two: bool) -> ModLoader {
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    if two {
        let provider = EventsMod::build("provider", &["provider"]);
        harness.install_mod("events", "events-provider", &provider, "events_provider.dll", 3);
    }
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    loader.enable_all();
    loader
}

#[test]
fn reads_give_the_values_of_the_stand_in_client() {
    let harness = start_in_world();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);

    assert_eq!(flag(table, client, AmmlClientFlag::WORLD_ACTIVE), (AmmlStatus::OK, 1));
    assert_eq!(flag(table, client, AmmlClientFlag::SHOWING_MENU), (AmmlStatus::OK, 0));
    assert_eq!(flag(table, client, AmmlClientFlag::HIDE_HUD), (AmmlStatus::OK, 0));
    assert_eq!(local_player(table, client), (AmmlStatus::OK, 0x5000));
    assert_eq!(flag(table, client, AmmlClientFlag(9)).0, AmmlStatus::INVALID_ARGUMENT);
    let mut current: *const c_void = std::ptr::null();
    // SAFETY: the table is live and `current` is writable.
    assert_eq!(unsafe { (table.current)(&mut current) }, AmmlStatus::OK);
    assert_eq!(current, client);
}

#[test]
fn local_player_is_null_while_no_world_is_joined() {
    let harness = start();
    let table = table(&harness);
    ClientGame::set_player(0x5000);

    assert_eq!(harness.state.events().gate().epoch().0, 0);
    assert_eq!(local_player(table, handle(&GAME_CLIENT_A)), (AmmlStatus::OK, 0));
    assert_eq!(flag(table, handle(&GAME_CLIENT_A), AmmlClientFlag::WORLD_ACTIVE), (AmmlStatus::OK, 1));
}

#[test]
fn top_screen_maps_names_and_frees_the_string_once() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);

    for (name, screen) in [
        ("pause_screen", AmmlScreen::PAUSE),
        ("hud_screen", AmmlScreen::HUD),
        ("hud_crosshair_screen", AmmlScreen::HUD),
        ("start_screen", AmmlScreen::START),
        ("ui://./hbui/index.html", AmmlScreen::START),
        ("world_loading_progress_screen - local_world_load", AmmlScreen::LOADING),
        ("toast_screen", AmmlScreen::TOAST),
        ("inventory_screen", AmmlScreen::CONTAINER),
        ("chat_screen", AmmlScreen::OTHER),
    ] {
        ClientGame::set_screen(name);
        let (_, frees) = HeapProbe::counts();
        assert_eq!(top_screen(table, client), (AmmlStatus::OK, screen), "{name}");
        assert_eq!(HeapProbe::counts().1, frees + 1, "{name}");
    }
}

#[test]
fn an_ore_ui_page_over_a_world_is_other() {
    let harness = start_in_world();
    let table = table(&harness);

    ClientGame::set_screen("ui://./hbui/index.html");

    assert_eq!(top_screen(table, handle(&GAME_CLIENT_A)), (AmmlStatus::OK, AmmlScreen::OTHER));
}

#[test]
fn an_ore_ui_page_after_a_leave_is_loading_until_the_save_screen_was_on_top() {
    let harness = start_in_world();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);

    SessionGame::exit();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_screen("ui://./hbui/index.html");
    assert_eq!(top_screen(table, client), (AmmlStatus::OK, AmmlScreen::LOADING));
    ClientGame::set_screen("world_saving_progress_screen - world_loading_progress_screen");
    assert_eq!(top_screen(table, client), (AmmlStatus::OK, AmmlScreen::LOADING));
    ClientGame::set_screen("ui://./hbui/index.html");

    assert_eq!(top_screen(table, client), (AmmlStatus::OK, AmmlScreen::START));
}

#[test]
fn a_read_from_another_thread_or_with_an_old_client_is_refused() {
    let harness = start_in_world();
    let table = table(&harness);
    let old = handle(&GAME_CLIENT_A) as usize;
    let before = ClientGame::calls();

    let other_thread = std::thread::spawn(move || {
        let client = std::ptr::with_exposed_provenance::<c_void>(old);
        [flag(table, client, AmmlClientFlag::WORLD_ACTIVE).0, top_screen(table, client).0]
    });
    assert_eq!(other_thread.join().unwrap(), [AmmlStatus::REFUSED; 2]);
    SessionGame::exit();
    ClientGame::update(&GAME_CLIENT_B);
    let client = std::ptr::with_exposed_provenance::<c_void>(old);
    assert_eq!(flag(table, client, AmmlClientFlag::WORLD_ACTIVE).0, AmmlStatus::REFUSED);
    assert_eq!(flag(table, client, AmmlClientFlag::HIDE_HUD).0, AmmlStatus::REFUSED);
    assert_eq!(top_screen(table, client).0, AmmlStatus::REFUSED);
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.open_pause_screen)(client) }, AmmlStatus::REFUSED);

    assert_eq!(ClientGame::calls(), before);
    assert_eq!(flag(table, handle(&GAME_CLIENT_B), AmmlClientFlag::WORLD_ACTIVE), (AmmlStatus::OK, 1));
}

#[test]
fn a_read_before_the_first_frame_is_not_ready() {
    let harness = install(&[]);
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);

    assert_eq!(flag(table, client, AmmlClientFlag::WORLD_ACTIVE).0, AmmlStatus::NOT_READY);
    assert_eq!(top_screen(table, client).0, AmmlStatus::NOT_READY);
    let mut current: *const c_void = std::ptr::null();
    // SAFETY: the table is live and `current` is writable.
    assert_eq!(unsafe { (table.current)(&mut current) }, AmmlStatus::NOT_READY);
    assert_eq!(ClientGame::calls(), [0; 9]);
}

#[test]
fn a_cancelled_grab_keeps_the_mouse_free() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    listen_grab(&harness, true);
    assert_eq!(flag(table, client, AmmlClientFlag::MOUSE_CAPTURED).0, AmmlStatus::NOT_AVAILABLE);

    ClientGame::game_release();
    ClientGame::game_grab();

    assert_eq!(ClientGame::grabs(), 0);
    assert_eq!(flag(table, client, AmmlClientFlag::MOUSE_CAPTURED), (AmmlStatus::OK, 0));
    assert_eq!(*GRABS_SEEN.lock().unwrap(), [format!("grab client={:#x}", client as usize)]);
}

#[test]
fn a_grab_the_listeners_let_through_captures_the_mouse() {
    let harness = start();
    let table = table(&harness);
    listen_grab(&harness, false);

    ClientGame::game_grab();

    assert_eq!(ClientGame::grabs(), 1);
    assert_eq!(GRABS_SEEN.lock().unwrap().len(), 1);
    assert_eq!(flag(table, handle(&GAME_CLIENT_A), AmmlClientFlag::MOUSE_CAPTURED), (AmmlStatus::OK, 1));
}

#[test]
fn the_loaders_own_grab_raises_no_mouse_grab_event() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    listen_grab(&harness, true);
    let adapter = ClientAdapter::get().unwrap();

    assert_eq!(adapter.set_flag(0, client, AmmlClientFlag::MOUSE_CAPTURED, true), Ok(()));
    assert_eq!(ClientGame::grabs(), 1);
    assert_eq!(flag(table, client, AmmlClientFlag::MOUSE_CAPTURED), (AmmlStatus::OK, 1));
    assert_eq!(adapter.set_flag(0, client, AmmlClientFlag::MOUSE_CAPTURED, false), Ok(()));

    assert!(GRABS_SEEN.lock().unwrap().is_empty());
    assert_eq!(ClientGame::calls()[6..8], [1, 1]);
    assert_eq!(flag(table, client, AmmlClientFlag::MOUSE_CAPTURED), (AmmlStatus::OK, 0));
    assert_eq!(adapter.set_flag(0, client, AmmlClientFlag::WORLD_ACTIVE, true), Err(AmmlStatus::INVALID_ARGUMENT));
}

#[test]
fn hud_is_restored_by_the_last_mod_on_the_next_tick() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    ClientGame::set_hide_hud(false);
    let loader = enable_mods(&harness, true);

    for module in ["@events/events", "@events/events-provider"] {
        assert_eq!(set_flag(table, module, client, AmmlClientFlag::HIDE_HUD, 1), AmmlStatus::OK);
    }
    assert_eq!(ClientGame::hud_sets(), [true, true]);
    loader.disable_one(0);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(ClientGame::hud_sets(), [true, true]);
    loader.disable_one(1);
    assert_eq!(ClientGame::hud_sets(), [true, true]);
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::update(&GAME_CLIENT_A);

    assert_eq!(ClientGame::hud_sets(), [true, true, false]);
    assert_eq!(flag(table, client, AmmlClientFlag::HIDE_HUD), (AmmlStatus::OK, 0));
}

#[test]
fn the_mouse_is_restored_when_it_still_holds_the_value_the_mod_set() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    ClientGame::game_release();
    let loader = enable_mods(&harness, false);
    assert_eq!(set_flag(table, "@events/events", client, AmmlClientFlag::MOUSE_CAPTURED, 1), AmmlStatus::OK);
    let [grabs, releases] = [ClientGame::calls()[6], ClientGame::calls()[7]];

    loader.disable_one(0);
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::update(&GAME_CLIENT_A);

    assert_eq!([ClientGame::calls()[6], ClientGame::calls()[7]], [grabs, releases + 1]);
    assert_eq!(flag(table, client, AmmlClientFlag::MOUSE_CAPTURED), (AmmlStatus::OK, 0));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "client flag restored");
    assert!(log.contains("MouseCaptured"), "{log}");
}

#[test]
fn the_mouse_is_left_when_the_game_changed_it_after_the_mod() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    ClientGame::game_release();
    let loader = enable_mods(&harness, false);
    assert_eq!(set_flag(table, "@events/events", client, AmmlClientFlag::MOUSE_CAPTURED, 1), AmmlStatus::OK);
    // A menu opens: the game releases the mouse the mod grabbed.
    ClientGame::game_release();
    let [grabs, releases] = [ClientGame::calls()[6], ClientGame::calls()[7]];

    loader.disable_one(0);
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::update(&GAME_CLIENT_A);

    assert_eq!([ClientGame::calls()[6], ClientGame::calls()[7]], [grabs, releases]);
    assert_eq!(flag(table, client, AmmlClientFlag::MOUSE_CAPTURED), (AmmlStatus::OK, 0));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "client flag left");
    assert!(log.contains("the game changed it after the mod set it"), "{log}");
}

#[test]
fn the_mouse_restore_follows_the_value_of_the_last_mod_released() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    ClientGame::game_grab();
    let loader = enable_mods(&harness, true);
    // Mod 0 releases the mouse, then mod 1 grabs it again: the grab holds mod 1's value, not mod 0's.
    assert_eq!(set_flag(table, "@events/events", client, AmmlClientFlag::MOUSE_CAPTURED, 0), AmmlStatus::OK);
    assert_eq!(set_flag(table, "@events/events-provider", client, AmmlClientFlag::MOUSE_CAPTURED, 1), AmmlStatus::OK);
    let [grabs, releases] = [ClientGame::calls()[6], ClientGame::calls()[7]];

    loader.disable_one(1);
    ClientGame::update(&GAME_CLIENT_A);
    loader.disable_one(0);
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::update(&GAME_CLIENT_A);

    // Mod 0, released last, set `false`; the grab is `true`, so the value from before (grabbed) is not written.
    assert_eq!([ClientGame::calls()[6], ClientGame::calls()[7]], [grabs, releases]);
    assert_eq!(flag(table, client, AmmlClientFlag::MOUSE_CAPTURED), (AmmlStatus::OK, 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "client flag left");
    assert!(log.contains("MouseCaptured"), "{log}");
}

#[test]
fn no_restore_runs_after_the_gate_closed() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    let loader = enable_mods(&harness, false);
    assert_eq!(set_flag(table, "@events/events", client, AmmlClientFlag::HIDE_HUD, 1), AmmlStatus::OK);

    harness.state.events().gate().close();
    loader.disable_one(0);
    ClientGame::update(&GAME_CLIENT_A);

    assert_eq!(ClientGame::hud_sets(), [true]);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "restore dropped");
    assert!(log.contains("the game does not run"), "{log}");
}

#[test]
fn open_pause_checks_the_listener_vtable_at_the_derived_offset() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);

    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.open_pause_screen)(client) }, AmmlStatus::OK);
    assert_eq!(ClientGame::focus_lost_this(), [ClientGame::game() + ClientGame::LISTENER_OFFSET]);
    ClientGame::place_listener(0x90);
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.open_pause_screen)(client) }, AmmlStatus::NOT_AVAILABLE);

    assert_eq!(ClientGame::focus_lost_this().len(), 1);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "pause screen not available");
    assert!(log.contains(r#""offset":"0x88""#), "{log}");
}

#[test]
fn open_pause_is_refused_when_the_option_is_off() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    ClientGame::set_pause_on_focus_lost(false);

    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.open_pause_screen)(client) }, AmmlStatus::REFUSED);
    assert!(ClientGame::focus_lost_this().is_empty());
    ClientGame::set_pause_on_focus_lost(true);
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.open_pause_screen)(client) }, AmmlStatus::OK);

    assert_eq!(ClientGame::focus_lost_this(), [ClientGame::game() + ClientGame::LISTENER_OFFSET]);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "pause option found");
    assert!(log.contains("pause screen refused: the game option is off"), "{log}");
    assert!(log.contains(r#""id":"0x32f""#), "{log}");
}

#[test]
fn open_pause_is_not_available_when_the_option_is_not_found() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    ClientGame::place_options(false);

    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.open_pause_screen)(client) }, AmmlStatus::NOT_AVAILABLE);

    assert!(ClientGame::focus_lost_this().is_empty());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "pause option not available");
    assert!(log.contains(r#""matches":0"#), "{log}");
}

#[test]
fn open_pause_is_not_available_when_two_options_carry_the_tag() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    ClientGame::add_option(0x500, "pause_menu_on_focus_lost");

    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.open_pause_screen)(client) }, AmmlStatus::NOT_AVAILABLE);

    assert!(ClientGame::focus_lost_this().is_empty());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "pause option not available");
    assert!(log.contains(r#""matches":2"#), "{log}");
}

#[test]
fn open_pause_is_not_available_when_an_option_does_not_hold_its_own_id() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    ClientGame::set_option_own_id(0x40, 0x41);

    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.open_pause_screen)(client) }, AmmlStatus::NOT_AVAILABLE);

    assert!(ClientGame::focus_lost_this().is_empty());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "does not hold its own id");
    assert!(log.contains(r#""found":65"#), "{log}");
}

#[test]
fn the_pause_area_needs_the_option_lookup() {
    let harness = install(&["OptionRegistry.getIfValid"]);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    let table = table(&harness);

    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.open_pause_screen)(handle(&GAME_CLIENT_A)) }, AmmlStatus::NOT_AVAILABLE);

    assert!(!harness.state.areas().is_available("client.pause", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "area not available");
    assert!(log.contains("client.pause") && log.contains("OptionRegistry.getIfValid"), "{log}");
}

#[test]
fn open_pause_releases_injected_input() {
    let harness = start();
    harness.state.areas().offer("input.key_inject", 1);
    let feeds = RecordingFeeds::new();
    feeds.bind(harness.state.input());
    feeds.listen(harness.state.events(), "input.injected_released", 0);
    let terms =
        LeaseTerms { mode: AmmlLeaseMode::SHARED, blocks: AmmlInputBlocks(0), passthrough: Vec::new(), preempt: false };
    let lease = harness.state.input().acquire(0, "@test/mod0", terms).unwrap();
    harness.state.input().key_down(lease, W).unwrap();
    ClientGame::frame();

    // SAFETY: the table is live.
    assert_eq!(unsafe { (table(&harness).open_pause_screen)(handle(&GAME_CLIENT_A)) }, AmmlStatus::OK);
    ClientGame::frame();
    ClientGame::frame();

    assert_eq!(
        feeds.log(),
        [
            "feed key 0x57 down".to_string(),
            format!("event released owner=0 lease={} reason={}", lease.0, AmmlReleaseReason::FOCUS_LOST.0),
            "feed key 0x57 up".to_string(),
        ]
    );
}

#[test]
fn focus_loss_releases_injected_input() {
    let harness = start();
    harness.state.areas().offer("input.key_inject", 1);
    let feeds = RecordingFeeds::new();
    feeds.bind(harness.state.input());
    feeds.listen(harness.state.events(), "input.injected_released", 0);
    let terms =
        LeaseTerms { mode: AmmlLeaseMode::SHARED, blocks: AmmlInputBlocks(0), passthrough: Vec::new(), preempt: false };
    let lease = harness.state.input().acquire(0, "@test/mod0", terms).unwrap();
    harness.state.input().key_down(lease, W).unwrap();
    ClientGame::frame();

    ClientGame::game_focus_lost();
    ClientGame::frame();
    ClientGame::frame();

    assert_eq!(
        feeds.log(),
        [
            "feed key 0x57 down".to_string(),
            format!("event released owner=0 lease={} reason={}", lease.0, AmmlReleaseReason::FOCUS_LOST.0),
            "feed key 0x57 up".to_string(),
        ]
    );
    assert_eq!(ClientGame::focus_lost_this(), [ClientGame::game() + ClientGame::LISTENER_OFFSET]);
}

#[test]
fn an_area_whose_entry_is_missing_is_not_offered() {
    let harness = install(&["IOptionRegistry.setHideGUI", "IMinecraftGame.releaseMouse"]);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    let areas = harness.state.areas();

    assert_eq!(flag(table, client, AmmlClientFlag::HIDE_HUD).0, AmmlStatus::NOT_AVAILABLE);
    assert_eq!(flag(table, client, AmmlClientFlag::MOUSE_CAPTURED).0, AmmlStatus::NOT_AVAILABLE);
    assert!(!areas.is_available("client.hud", 1) && !areas.is_available("client.mouse", 1));
    assert!(!harness.state.events().is_available("client.mouse_grab", 1));
    assert!(areas.is_available("client.read", 1) && areas.is_available("client.pause", 1));
    ClientGame::game_grab();
    assert_eq!(ClientGame::grabs(), 1);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "area not available");
    assert!(log.contains("client.hud") && log.contains("IOptionRegistry.setHideGUI"), "{log}");
    assert!(log.contains("area adapter"), "{log}");
}

// The session and client adapters over the any-input stand-ins without `without`, after one frame and one update
// of client A.
fn start_activity(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&ActivityGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

fn note_activity(table: &AmmlClientTable, client: *const c_void) -> AmmlStatus {
    // SAFETY: the loader checks the handle before it calls into the stand-ins.
    unsafe { table.note_activity.unwrap()(client) }
}

#[test]
fn note_activity_runs_the_any_input_handler_with_the_client_and_gives_the_use_back() {
    let harness = start_activity(&[]);
    let uses = ActivityGame::uses();

    assert_eq!(note_activity(table(&harness), handle(&GAME_CLIENT_A)), AmmlStatus::OK);

    assert_eq!(ActivityGame::calls(), [ClientGame::client(&GAME_CLIENT_A) as usize]);
    assert_eq!(ActivityGame::uses(), uses);
}

#[test]
fn note_activity_is_refused_while_the_game_is_not_ready_and_calls_nothing() {
    let harness = start_activity(&[]);
    let uses = ActivityGame::uses();
    let table = table(&harness);

    ActivityGame::set_valid(false);
    assert_eq!(note_activity(table, handle(&GAME_CLIENT_A)), AmmlStatus::REFUSED);
    ActivityGame::set_valid(true);
    ActivityGame::set_input_handler(false);
    assert_eq!(note_activity(table, handle(&GAME_CLIENT_A)), AmmlStatus::REFUSED);
    ActivityGame::set_input_handler(true);
    ActivityGame::set_handler(false);
    assert_eq!(note_activity(table, handle(&GAME_CLIENT_A)), AmmlStatus::REFUSED);

    assert!(ActivityGame::calls().is_empty());
    assert_eq!(ActivityGame::uses(), uses);
}

#[test]
fn note_activity_for_another_client_is_refused() {
    let harness = start_activity(&[]);

    assert_eq!(note_activity(table(&harness), handle(&GAME_CLIENT_B)), AmmlStatus::REFUSED);
    assert!(ActivityGame::calls().is_empty());
}

#[test]
fn note_activity_without_a_derived_offset_is_not_available() {
    let harness = start_activity(&["InputHandler.mAnyInputHandler"]);

    assert_eq!(note_activity(table(&harness), handle(&GAME_CLIENT_A)), AmmlStatus::NOT_AVAILABLE);
    assert!(ActivityGame::calls().is_empty());
}
