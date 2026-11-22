use std::ffi::c_void;

use amml::{ClientAdapter, ModLoader, OptionsAdapter, SessionAdapter};
use amml_abi::{AmmlApi, AmmlOptionKey, AmmlOptionsTable, AmmlPersistence, AmmlServices, AmmlStatus, AmmlStr};
use amml_runtime::ModRegistry;

use crate::support::{ClientGame, EventsMod, FakeGameClient, GAME_CLIENT_A, GAME_CLIENT_B, Harness, LogFile};

const EVENTS: &str = "@events/events";
const PROVIDER: &str = "@events/events-provider";

// The session, client and options adapters over every stand-in entry; after one frame and one update of client A, which
// makes it the primary client.
fn start() -> Harness {
    let harness = Harness::start_with_entries(&ClientGame::entries(&[]));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    OptionsAdapter::install(harness.state);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

fn table(harness: &Harness) -> AmmlOptionsTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::options(services) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlOptionsTable::read(table) }.unwrap()
}

fn get_int(table: &AmmlOptionsTable, key: AmmlOptionKey) -> (AmmlStatus, i32) {
    let mut out = -77;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.get_int)(key, &mut out) };
    (status, out)
}

fn set_int(table: &AmmlOptionsTable, module: &str, key: AmmlOptionKey, value: i32) -> AmmlStatus {
    // SAFETY: the table is live and the view lives for the call.
    unsafe { (table.set_int)(AmmlStr::from_str(module), key, value, AmmlPersistence::TRANSIENT) }
}

fn release(table: &AmmlOptionsTable, module: &str, key: AmmlOptionKey) -> AmmlStatus {
    // SAFETY: the table is live and the view lives for the call.
    unsafe { (table.release_transient)(AmmlStr::from_str(module), key) }
}

fn view_distance(table: &AmmlOptionsTable, client: *const c_void) -> (AmmlStatus, u32) {
    let mut out = 0xEE;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.view_distance_chunks)(client, &mut out) };
    (status, out)
}

fn handle(client: &'static FakeGameClient) -> *const c_void {
    ClientGame::client(client).cast_const()
}

// Loads and enables the events-mod fixture variants `plain` (index 0) and `provider` (index 1).
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

#[test]
fn view_distance_comes_from_the_registry_of_the_primary_client() {
    let harness = start();
    let table = table(&harness);
    ClientGame::set_view_distance(18);

    assert_eq!(view_distance(&table, handle(&GAME_CLIENT_A)), (AmmlStatus::OK, 18));
    assert_eq!(view_distance(&table, handle(&GAME_CLIENT_B)).0, AmmlStatus::REFUSED);
    // SAFETY: the table is live; a null out is refused before it is written.
    let null = unsafe { (table.view_distance_chunks)(handle(&GAME_CLIENT_A), std::ptr::null_mut()) };
    assert_eq!(null, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn the_int_options_are_read_and_their_ids_and_tags_are_logged_once() {
    let harness = start();
    let table = table(&harness);

    assert_eq!(get_int(&table, AmmlOptionKey::VSYNC), (AmmlStatus::OK, 1));
    assert_eq!(get_int(&table, AmmlOptionKey::MAX_FRAMERATE), (AmmlStatus::OK, 60));
    assert_eq!(get_int(&table, AmmlOptionKey::VSYNC).0, AmmlStatus::OK);
    assert_eq!(get_int(&table, AmmlOptionKey(9)).0, AmmlStatus::INVALID_ARGUMENT);

    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "option key");
    assert_eq!(log.matches("option key").count(), 2, "{log}");
    assert!(log.contains("gfx_vsync") && log.contains("gfx_max_framerate"), "{log}");
}

#[test]
fn vsync_scan_needs_exactly_one_match() {
    let harness = start();
    let table = table(&harness);
    let _loader = enable_mods(&harness);
    ClientGame::add_option(0x500, "gfx_vsync");

    assert_eq!(get_int(&table, AmmlOptionKey::VSYNC).0, AmmlStatus::NOT_AVAILABLE);
    // The area is taken back: this build does not offer `options.int`.
    assert!(!harness.state.areas().is_available("options.int", 1));
    assert_eq!(get_int(&table, AmmlOptionKey::MAX_FRAMERATE).0, AmmlStatus::NOT_AVAILABLE);
    assert_eq!(set_int(&table, EVENTS, AmmlOptionKey::VSYNC, 0), AmmlStatus::NOT_AVAILABLE);
    assert!(ClientGame::int_sets().is_empty());
    // The view distance does not depend on the scan.
    assert!(harness.state.areas().is_available("options.view_distance", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "options.int not available");
    assert!(log.contains("matches"), "{log}");
}

#[test]
fn off_the_game_thread_is_refused() {
    let harness = start();
    let table = table(&harness);
    let _loader = enable_mods(&harness);
    let client = handle(&GAME_CLIENT_A).addr();

    let results = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                [
                    view_distance(&table, std::ptr::with_exposed_provenance(client)).0,
                    get_int(&table, AmmlOptionKey::VSYNC).0,
                    set_int(&table, EVENTS, AmmlOptionKey::VSYNC, 0),
                    release(&table, EVENTS, AmmlOptionKey::VSYNC),
                ]
            })
            .join()
            .unwrap()
    });

    assert_eq!(results, [AmmlStatus::REFUSED; 4]);
    assert!(ClientGame::int_sets().is_empty());
    assert_eq!(ClientGame::int_option(ClientGame::VSYNC_ID), 1);
}

#[test]
fn transient_values_restore_keep_and_refuse_through_the_table() {
    let harness = start();
    let table = table(&harness);
    let _loader = enable_mods(&harness);
    let vsync = ClientGame::VSYNC_ID;

    assert_eq!(set_int(&table, EVENTS, AmmlOptionKey::VSYNC, 0), AmmlStatus::OK);
    assert_eq!(get_int(&table, AmmlOptionKey::VSYNC), (AmmlStatus::OK, 0));
    assert_eq!(ClientGame::int_sets(), [(vsync, 0, false)]);
    // Another mod may not take or release the key.
    assert_eq!(set_int(&table, PROVIDER, AmmlOptionKey::VSYNC, 2), AmmlStatus::REFUSED);
    assert_eq!(release(&table, PROVIDER, AmmlOptionKey::VSYNC), AmmlStatus::REFUSED);
    // A second set of the owner moves the value but keeps the value from before the first set.
    assert_eq!(set_int(&table, EVENTS, AmmlOptionKey::VSYNC, 2), AmmlStatus::OK);
    assert_eq!(release(&table, EVENTS, AmmlOptionKey::VSYNC), AmmlStatus::OK);
    assert_eq!(ClientGame::int_option(vsync), 1);
    assert_eq!(ClientGame::int_sets().last(), Some(&(vsync, 1, true)));

    // A value the user set while the transient value was set stays.
    assert_eq!(set_int(&table, EVENTS, AmmlOptionKey::VSYNC, 0), AmmlStatus::OK);
    ClientGame::set_int_option(vsync, 2);
    let writes = ClientGame::int_sets().len();
    assert_eq!(release(&table, EVENTS, AmmlOptionKey::VSYNC), AmmlStatus::OK);
    assert_eq!(ClientGame::int_sets().len(), writes);
    assert_eq!(ClientGame::int_option(vsync), 2);
    // Nothing is held now: a release writes nothing.
    assert_eq!(release(&table, EVENTS, AmmlOptionKey::VSYNC), AmmlStatus::OK);
    assert_eq!(ClientGame::int_sets().len(), writes);
    // Only a transient set exists; any other persistence is refused.
    // SAFETY: the table is live and the view lives for the call.
    let status = unsafe { (table.set_int)(AmmlStr::from_str(EVENTS), AmmlOptionKey::VSYNC, 0, AmmlPersistence(9)) };
    assert_eq!(status, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn max_framerate_restores_the_option_it_set_when_the_effective_option_changes() {
    let harness = start();
    let table = table(&harness);
    let _loader = enable_mods(&harness);
    let (first, other) = (ClientGame::MAX_FRAMERATE_ID, ClientGame::MAX_FRAMERATE_OTHER_ID);
    assert_eq!(set_int(&table, EVENTS, AmmlOptionKey::MAX_FRAMERATE, 0), AmmlStatus::OK);
    assert_eq!(ClientGame::int_sets(), [(first, 0, false)]);

    // The registry's slot 13 now answers 2: the effective option is F7h.
    ClientGame::set_framerate_slot_13(2);
    // Reads follow the effective option.
    assert_eq!(get_int(&table, AmmlOptionKey::MAX_FRAMERATE), (AmmlStatus::OK, 30));

    // The restore follows the option the mod set.
    assert_eq!(release(&table, EVENTS, AmmlOptionKey::MAX_FRAMERATE), AmmlStatus::OK);
    assert_eq!((ClientGame::int_option(first), ClientGame::int_option(other)), (60, 30));
    assert_eq!(ClientGame::int_sets().last(), Some(&(first, 60, true)));
}

#[test]
fn disable_releases_the_transient_values_of_that_mod() {
    let harness = start();
    let table = table(&harness);
    let loader = enable_mods(&harness);
    let (vsync, limit) = (ClientGame::VSYNC_ID, ClientGame::MAX_FRAMERATE_ID);
    assert_eq!(set_int(&table, EVENTS, AmmlOptionKey::VSYNC, 0), AmmlStatus::OK);
    assert_eq!(set_int(&table, EVENTS, AmmlOptionKey::MAX_FRAMERATE, 0), AmmlStatus::OK);
    assert_eq!(set_int(&table, PROVIDER, AmmlOptionKey::MAX_FRAMERATE, 0), AmmlStatus::REFUSED);

    loader.disable_one(0);

    assert_eq!((ClientGame::int_option(vsync), ClientGame::int_option(limit)), (1, 60));
    let sets = ClientGame::int_sets();
    assert_eq!(&sets[2..], [(vsync, 1, true), (limit, 60, true)]);
    // The key is free for another mod after the disable.
    assert_eq!(set_int(&table, PROVIDER, AmmlOptionKey::VSYNC, 2), AmmlStatus::OK);
}
