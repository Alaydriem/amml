use std::sync::Arc;

use amml::{ModLoader, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlInputBlocks, AmmlInputTable, AmmlLeaseMode, AmmlLeaseRequest, AmmlLeaseState, AmmlReleaseReason,
    AmmlServices, AmmlStatus, AmmlStr,
};
use amml_runtime::{LeaseTerms, ModRegistry};

use crate::support::{CLIENT_A, EventsMod, Harness, RecordingFeeds, SessionGame};

const W: u16 = 0x57;

// A harness with the session adapter over the stand-in game, after one frame (the game gate is open).
fn start() -> Harness {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    SessionGame::frame();
    harness
}

// The input table, through the services table as a mod reaches it.
fn input(harness: &Harness) -> &'static AmmlInputTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::input(services) }.unwrap();
    // SAFETY: as above.
    unsafe { &*table }
}

// Offers key injection and binds the recording feeds.
fn bind_keys(harness: &Harness) -> Arc<RecordingFeeds> {
    harness.state.areas().offer("input.key_inject", 1);
    let feeds = RecordingFeeds::new();
    feeds.bind(harness.state.input());
    feeds
}

fn lease(harness: &Harness, owner: usize, preempt: bool) -> u64 {
    let terms =
        LeaseTerms { mode: AmmlLeaseMode::SHARED, blocks: AmmlInputBlocks(0), passthrough: Vec::new(), preempt };
    harness.state.input().acquire(owner, &format!("@test/mod{owner}"), terms).unwrap().0
}

fn key_down(table: &AmmlInputTable, lease: u64, key: u16) -> (AmmlStatus, u64) {
    let mut hold = 0u64;
    // SAFETY: the table is live and `hold` is writable.
    let status = unsafe { (table.key_down)(lease, key, &mut hold) };
    (status, hold)
}

#[test]
fn injection_before_the_keys_adapter_binds_is_not_ready() {
    let harness = start();
    harness.state.areas().offer("input.key_inject", 1);
    let table = input(&harness);
    let lease = lease(&harness, 0, false);

    assert_eq!(key_down(table, lease, W).0, AmmlStatus::NOT_READY);
}

#[test]
fn injection_on_a_build_without_the_area_is_not_available() {
    let harness = start();
    let feeds = RecordingFeeds::new();
    feeds.bind(harness.state.input());
    let table = input(&harness);
    let lease = lease(&harness, 0, false);

    assert_eq!(key_down(table, lease, W).0, AmmlStatus::NOT_AVAILABLE);
    SessionGame::frame();
    assert!(feeds.log().is_empty());
}

#[test]
fn a_suspended_lease_refuses_injection() {
    let harness = start();
    let _feeds = bind_keys(&harness);
    let table = input(&harness);
    let first = lease(&harness, 0, false);
    let _preempt = lease(&harness, 1, true);

    assert_eq!(key_down(table, first, W).0, AmmlStatus::REFUSED);
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.lease_state)(first) }, AmmlLeaseState::SUSPENDED);
}

#[test]
fn an_injected_key_reaches_the_feed_once_in_the_next_frame() {
    let harness = start();
    let feeds = bind_keys(&harness);
    feeds.listen(harness.state.events(), "input.key", 0);
    feeds.listen(harness.state.events(), "input.key", 5);
    let table = input(&harness);
    let lease = lease(&harness, 0, false);

    let (status, _) = key_down(table, lease, W);
    assert_eq!(status, AmmlStatus::OK);
    assert!(feeds.log().is_empty());
    SessionGame::frame();
    SessionGame::frame();

    assert_eq!(
        feeds.log(),
        [
            "event key owner=0 key=0x57 down=1 injected=1 origin=1",
            "event key owner=5 key=0x57 down=1 injected=1 origin=2",
            "feed key 0x57 down",
        ]
    );
}

#[test]
fn disable_releases_the_mods_holds_and_tells_no_one_else() {
    let harness = start();
    let feeds = bind_keys(&harness);
    let plain = EventsMod::build("plain", &[]);
    let provider = EventsMod::build("provider", &["provider"]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    harness.install_mod("events", "events-provider", &provider, "events_provider.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    loader.enable_all();
    let hub = harness.state.events();
    for owner in [0, 1] {
        feeds.listen(hub, "input.injected_released", owner);
        feeds.listen(hub, "input.lease_resumed", owner);
    }
    let table = input(&harness);
    let request = AmmlLeaseRequest {
        size: AmmlLeaseRequest::SIZE_V1,
        mode: AmmlLeaseMode::SHARED,
        preempt: 0,
        _reserved: [0; 2],
        blocks: AmmlInputBlocks(0),
        _reserved2: 0,
        passthrough: std::ptr::null(),
        passthrough_count: 0,
    };
    let mut leases = [0u64; 2];
    for (module, out) in ["@events/events", "@events/events-provider"].iter().zip(leases.iter_mut()) {
        // SAFETY: the view, the request and the output live for the call.
        let status = unsafe { (table.acquire)(AmmlStr::from_str(module), &request, out) };
        assert_eq!(status, AmmlStatus::OK);
    }
    assert_eq!(key_down(table, leases[0], W).0, AmmlStatus::OK);
    SessionGame::frame();

    loader.disable_one(0);
    // The disabled mod's listeners are gone, so a notice for it would never show in the log.
    assert_eq!(harness.state.owner_events().queued_for(0), 0);
    SessionGame::frame();
    SessionGame::frame();

    assert_eq!(feeds.log(), ["feed key 0x57 down", "feed key 0x57 up"]);
    // SAFETY: the table is live.
    let states = unsafe { [(table.lease_state)(leases[0]), (table.lease_state)(leases[1])] };
    assert_eq!(states, [AmmlLeaseState::LOST, AmmlLeaseState::ACTIVE]);
}

#[test]
fn world_left_releases_held_keys_with_reason_world_left() {
    let harness = start();
    let feeds = bind_keys(&harness);
    feeds.listen(harness.state.events(), "input.injected_released", 0);
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    let table = input(&harness);
    let lease = lease(&harness, 0, false);
    assert_eq!(key_down(table, lease, W).0, AmmlStatus::OK);
    SessionGame::frame();

    SessionGame::exit();
    assert_eq!(feeds.log(), ["feed key 0x57 down"]);
    SessionGame::frame();
    SessionGame::frame();

    assert_eq!(
        feeds.log(),
        [
            "feed key 0x57 down".to_string(),
            format!("event released owner=0 lease={lease} reason={}", AmmlReleaseReason::WORLD_LEFT.0),
            "feed key 0x57 up".to_string(),
        ]
    );
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.lease_state)(lease) }, AmmlLeaseState::ACTIVE);
}

#[test]
fn a_released_hold_can_be_released_again() {
    let harness = start();
    let feeds = bind_keys(&harness);
    let table = input(&harness);
    let lease = lease(&harness, 0, false);
    let (_, hold) = key_down(table, lease, W);
    SessionGame::frame();

    // SAFETY: the table is live.
    let statuses = unsafe { [(table.release_hold)(hold), (table.release_hold)(hold)] };
    SessionGame::frame();
    SessionGame::frame();

    assert_eq!(statuses, [AmmlStatus::OK, AmmlStatus::OK]);
    assert_eq!(feeds.log(), ["feed key 0x57 down", "feed key 0x57 up"]);
}

#[test]
fn an_unknown_lease_is_lost() {
    let harness = start();
    let table = input(&harness);

    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.lease_state)(12345) }, AmmlLeaseState::LOST);
}

#[test]
fn is_down_of_a_key_outside_the_table_is_an_invalid_argument() {
    let harness = start();
    let table = input(&harness);
    harness.state.input().keys().real(W, true);
    let mut out = 9u8;

    // SAFETY: the table is live and `out` is writable.
    let (outside, inside) = unsafe { ((table.is_down)(0x100, &mut out), (table.is_down)(W, &mut out)) };

    assert_eq!((outside, inside, out), (AmmlStatus::INVALID_ARGUMENT, AmmlStatus::OK, 1));
}

#[test]
fn coexist_pairs_need_an_even_count_and_a_live_mod() {
    let harness = start();
    let table = input(&harness);
    let set_coexist = table.set_coexist.unwrap();
    let ids = [AmmlStr::from_str("@a/x"), AmmlStr::from_str("@b/y"), AmmlStr::from_str("@c/z")];

    // SAFETY: the views live for the calls.
    let odd = unsafe { set_coexist(AmmlStr::from_str("@test/none"), ids.as_ptr(), 3) };
    // SAFETY: as above.
    let unknown = unsafe { set_coexist(AmmlStr::from_str("@test/none"), ids.as_ptr(), 2) };

    assert_eq!(odd, AmmlStatus::INVALID_ARGUMENT);
    assert_ne!(unknown, AmmlStatus::OK);
    assert!(table.size >= AmmlInputTable::SIZE_V2);
}
