use std::ffi::c_void;
use std::sync::Mutex;

use amml::{ModListener, ModLoader, ModPhase, OwnerEvent, SessionAdapter};
use amml_abi::{
    AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlStatus, GameTick, InputLeaseSuspended, WorldJoined, WorldLeft,
};
use amml_runtime::{ListenError, ModRegistry, ResourceError, ResourceKind, WorldEpoch};

use crate::support::{CLIENT_A, CLIENT_B, EventsMod, FakeClient, Harness, LogFile, SessionGame};

// The events a test listens to; a listener's `user` value is its index here.
const EVENTS: [&str; 6] =
    ["frame.update", "client.tick", "client.game_tick", "world.join_started", "world.joined", "world.left"];

static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

// Records `<event> phase=<p> epoch=<context epoch> client=<context client>`, plus the payload of the events that have one.
unsafe extern "C" fn record(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    let name = EVENTS[user as usize];
    // SAFETY: the loader passes a live header and context for the call.
    let (header, context) = unsafe { (&*event, &*context) };
    let payload = match name {
        // SAFETY: the loader raises each of these events with its own struct.
        "client.game_tick" => format!(" tick={}", unsafe { (*event.cast::<GameTick>()).tick }),
        // SAFETY: as above.
        "world.joined" => format!(" world={}", unsafe { (*event.cast::<WorldJoined>()).epoch }),
        // SAFETY: as above.
        "world.left" => format!(" world={}", unsafe { (*event.cast::<WorldLeft>()).epoch }),
        _ => String::new(),
    };
    let line = format!(
        "{name} phase={} epoch={} client={:#x}{payload}",
        header.phase.0, context.epoch, context.client as usize
    );
    SEEN.lock().unwrap().push(line);
    AmmlStatus::OK
}

// Installs the adapter over every session entry but `without` and listens to every offered event.
fn start(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&SessionGame::entries(without));
    SessionAdapter::install(harness.state);
    let hub = harness.state.events();
    for (index, event) in EVENTS.iter().enumerate() {
        let listener = ModListener { owner: 9, callback: record, user: index };
        let _ = hub.listen(event, 1, AmmlBand::NORMAL, listener);
    }
    harness
}

// Records `mod <phase>` for a listener of the first mod.
unsafe extern "C" fn record_mod(
    event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    _user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live header for the call.
    let phase = unsafe { (*event).phase.0 };
    SEEN.lock().unwrap().push(format!("mod {phase}"));
    AmmlStatus::OK
}

// Installs the events-mod fixture as the first mod, then loads and enables it.
fn start_events_mod(harness: &Harness) -> ModLoader {
    let dll = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &dll, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    loader.enable_all();
    loader
}

fn seen(prefix: &str) -> Vec<String> {
    SEEN.lock().unwrap().iter().filter(|line| line.starts_with(prefix)).cloned().collect()
}

fn address(client: &'static FakeClient) -> String {
    format!("{:#x}", SessionGame::client(client) as usize)
}

#[test]
fn frame_update_opens_the_game_gate() {
    let harness = start(&[]);
    assert!(!harness.state.events().gate().is_open());

    SessionGame::frame();

    assert!(harness.state.events().gate().is_open());
    assert_eq!(
        seen("frame.update"),
        ["frame.update phase=0 epoch=0 client=0x0", "frame.update phase=1 epoch=0 client=0x0"]
    );
    assert_eq!(SessionGame::calls()[0], 1);
}

#[test]
fn world_joined_comes_once_per_join_with_a_new_epoch() {
    let _harness = start(&[]);
    let client = address(&CLIENT_A);
    SessionGame::set_player(0x5000);

    SessionGame::start_join(&CLIENT_A, [0; 7]);
    for _ in 0..3 {
        SessionGame::update(&CLIENT_A);
    }
    SessionGame::exit();
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);

    assert_eq!(
        seen("world.joined"),
        [
            format!("world.joined phase=1 epoch=1 client={client} world=1"),
            format!("world.joined phase=1 epoch=2 client={client} world=2"),
        ]
    );
    assert_eq!(seen("world.left"), [format!("world.left phase=0 epoch=1 client={client} world=1")]);
    assert_eq!(seen("world.join_started").len(), 2);
    assert_eq!(SessionGame::calls(), [0, 4, 0, 2, 1]);
}

#[test]
fn a_mod_queued_for_disable_is_disabled_before_the_frame_update_listeners_run() {
    let harness = start(&[]);
    let loader = start_events_mod(&harness);
    let hub = harness.state.events();
    let listener = ModListener { owner: 0, callback: record_mod, user: 0 };
    hub.listen("frame.update", 1, AmmlBand::NORMAL, listener).unwrap();
    hub.deferred().push(0);

    SessionGame::frame();

    assert_eq!(loader.phases(), [("@events/events".to_string(), ModPhase::Disabled)]);
    assert!(seen("mod").is_empty());
    assert_eq!(seen("frame.update").len(), 2);
}

#[test]
fn a_level_exit_before_the_local_player_keeps_the_join_waiting() {
    let _harness = start(&[]);
    let client = address(&CLIENT_A);

    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    SessionGame::exit();
    SessionGame::set_player(0x5000);
    SessionGame::update(&CLIENT_A);

    assert_eq!(seen("world.joined"), [format!("world.joined phase=1 epoch=1 client={client} world=1")]);
    assert!(seen("world.left").is_empty());
}

#[test]
fn a_client_with_a_null_vtable_never_joins() {
    let _harness = start(&[]);
    SessionGame::set_player(0x5000);

    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update_null_vtable();
    SessionGame::update_null_vtable();

    assert!(seen("world.joined").is_empty());
    assert_eq!(seen("client.tick").len(), 2);
    assert_eq!(SessionGame::calls()[1], 2);
}

#[test]
fn client_update_passes_whole_registers_through() {
    let _harness = start(&[]);

    SessionGame::update_raw(SessionGame::client(&CLIENT_A), 0x1234_5678_9ABC_DE01, 0xFEED_0000_0000_0001);

    assert_eq!(SessionGame::update_args(), (0x1234_5678_9ABC_DE01, 0xFEED_0000_0000_0001));
}

#[test]
fn after_a_leave_the_epoch_is_zero_and_its_resources_outlive_the_next_leave() {
    let harness = start(&[]);
    let gate = harness.state.events().gate();
    let resources = harness.state.resources();
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    SessionGame::exit();
    assert_eq!(gate.epoch(), WorldEpoch(0));
    let id = resources.insert(ResourceKind(1), 0, gate.epoch(), 7).unwrap();

    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    SessionGame::exit();

    assert_eq!(resources.get(id), Ok(7));
    assert_eq!(seen("world.left").len(), 2);
    assert_eq!(gate.epoch(), WorldEpoch(0));
}

#[test]
fn world_left_is_offered_only_with_world_joined() {
    let harness = start(&["ClientInstance.getLocalPlayer"]);
    let hub = harness.state.events();

    assert!(!hub.is_available("world.joined", 1));
    assert!(!hub.is_available("world.left", 1));
    assert!(hub.is_available("client.tick", 1));
}

#[test]
fn world_joined_waits_for_the_local_player() {
    let _harness = start(&[]);

    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    SessionGame::set_player(0x5000);
    SessionGame::update(&CLIENT_A);

    assert_eq!(seen("world.joined").len(), 1);
    assert_eq!(seen("client.tick").len(), 2);
}

#[test]
fn world_left_frees_resources_of_that_world() {
    let harness = start(&[]);
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    let resources = harness.state.resources();
    let id = resources.insert(ResourceKind(1), 0, WorldEpoch(1), 7).unwrap();
    assert_eq!(resources.get(id), Ok(7));

    SessionGame::exit();

    assert_eq!(resources.get(id), Err(ResourceError::Stale));
    assert_eq!(SessionGame::calls()[4], 1);
}

#[test]
fn a_resource_made_with_the_epoch_of_a_left_world_is_refused() {
    let harness = start(&[]);
    let gate = harness.state.events().gate();
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    // An area call on another thread read the epoch while the world was still joined.
    let read_before_leave = gate.epoch();
    assert_ne!(read_before_leave, WorldEpoch(0));

    SessionGame::exit();

    let late = harness.state.resources().insert(ResourceKind(1), 0, read_before_leave, 7);
    assert_eq!(late, Err((ResourceError::Stale, 7)));
}

#[test]
fn a_second_client_does_not_raise_client_tick() {
    let _harness = start(&[]);
    let client = address(&CLIENT_A);

    SessionGame::update(&CLIENT_A);
    SessionGame::update(&CLIENT_B);
    SessionGame::update(&CLIENT_A);

    assert_eq!(seen("client.tick"), vec![format!("client.tick phase=1 epoch=0 client={client}"); 2]);
    assert_eq!(SessionGame::calls()[1], 3);
}

#[test]
fn game_tick_carries_the_tick_and_join_started_passes_every_argument_through() {
    let _harness = start(&[]);
    // rdx is the flag; r8, r9 and the stack word at entry+28h are the addresses of the three strings, null here.
    let args = [0x11, 0, 0, 0, 0x55, 0x66, 0x77];

    SessionGame::frame();
    SessionGame::update(&CLIENT_A);
    SessionGame::tick(42);
    SessionGame::start_join(&CLIENT_A, args);

    let client = format!("{:#x}", SessionGame::client(&CLIENT_A) as usize);
    assert_eq!(seen("client.game_tick"), [format!("client.game_tick phase=1 epoch=0 client={client} tick=42")]);
    assert_eq!(seen("world.join_started"), ["world.join_started phase=1 epoch=0 client=0x0"]);
    assert_eq!(SessionGame::join_args(), args);
    assert_eq!(SessionGame::calls()[2..4], [1, 1]);
}

#[test]
fn game_tick_is_raised_only_for_the_primary_client() {
    let _harness = start(&[]);

    SessionGame::frame();
    SessionGame::tick(1);
    SessionGame::update(&CLIENT_A);
    SessionGame::tick_on(&CLIENT_B, 2);
    SessionGame::tick_on(&CLIENT_A, 3);

    let client = format!("{:#x}", SessionGame::client(&CLIENT_A) as usize);
    assert_eq!(seen("client.game_tick"), [format!("client.game_tick phase=1 epoch=0 client={client} tick=3")]);
    assert_eq!(SessionGame::calls()[2], 3);
}

#[test]
fn an_event_whose_entry_is_missing_is_not_offered() {
    let harness = start(&["ClientInstance.onTick", "ClientInstance.update"]);
    let hub = harness.state.events();
    let listener = ModListener { owner: 9, callback: record, user: 2 };

    start_events_mod(&harness);

    assert_eq!(hub.listen("client.game_tick", 1, AmmlBand::NORMAL, listener).err(), Some(ListenError::UnknownEvent));
    let listen = EventsMod::lines_of(&harness.mods(), "events", "listen client.tick");
    assert_eq!(listen, [format!("events listen client.tick status={}", AmmlStatus::NOT_AVAILABLE.0)]);
    assert!(hub.is_available("frame.update", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "event not available");
    assert!(log.contains("client.game_tick") && log.contains("ClientInstance.onTick"), "{log}");
    assert!(log.contains("event offered"), "{log}");
}

// Records `owner lease=<lease>` for the lease events of the owner whose id is the `user` value.
unsafe extern "C" fn record_owner_event(
    event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `input.lease_suspended` with an `InputLeaseSuspended`.
    let lease = unsafe { (*event.cast::<InputLeaseSuspended>()).lease };
    SEEN.lock().unwrap().push(format!("owner {} lease={lease}", user as usize));
    AmmlStatus::OK
}

#[test]
fn owner_events_are_delivered_on_the_next_frame() {
    let harness = start(&[]);
    let hub = harness.state.events();
    hub.listen(
        "input.lease_suspended",
        1,
        AmmlBand::NORMAL,
        ModListener { owner: 4, callback: record_owner_event, user: 4 },
    )
    .unwrap();
    hub.listen(
        "input.lease_suspended",
        1,
        AmmlBand::NORMAL,
        ModListener { owner: 5, callback: record_owner_event, user: 5 },
    )
    .unwrap();

    std::thread::scope(|scope| {
        scope.spawn(|| harness.state.owner_events().push(4, OwnerEvent::Suspended { lease: 7 }));
    });
    assert!(SEEN.lock().unwrap().is_empty());
    SessionGame::frame();
    SessionGame::frame();

    let lines = SEEN.lock().unwrap().clone();
    assert_eq!(lines[0], "owner 4 lease=7");
    assert_eq!(lines.iter().filter(|line| line.starts_with("owner")).count(), 1);
    assert!(lines[1].starts_with("frame.update"));
}

#[test]
fn the_game_thread_is_the_thread_of_the_frame() {
    let harness = start(&[]);
    let gate = harness.state.events().gate();
    assert!(!gate.on_game_thread());

    SessionGame::frame();

    assert!(gate.on_game_thread());
    std::thread::scope(|scope| {
        assert!(!scope.spawn(|| gate.on_game_thread()).join().unwrap());
    });
}

#[test]
fn the_adapter_names_the_primary_client_and_the_game() {
    let _harness = start(&[]);
    assert_eq!((SessionAdapter::primary_client(), SessionAdapter::minecraft_game()), (None, None));

    SessionGame::frame();
    SessionGame::update(&CLIENT_A);

    assert_eq!(SessionAdapter::primary_client(), Some(SessionGame::client(&CLIENT_A) as usize));
    assert_eq!(SessionAdapter::minecraft_game(), Some(0x6A_0000));
}
