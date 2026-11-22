use std::ffi::c_void;

use amml::{ClientAdapter, SessionActions, SessionAdapter};
use amml_abi::{AmmlApi, AmmlClientTable, AmmlServices, AmmlStartWorldOutcome, AmmlStatus, AmmlStr};
use amml_data::{EntryKind, Resolved};

use crate::support::{ClientGame, DirectJoinGame, FakeGameClient, GAME_CLIENT_A, Harness, LogFile, WorldGame};

// The entries of the stand-in client plus those of `client.leave_world`, `client.send_command` and
// `client.start_local_world`. The values are never called: every call here is refused before the game.
fn entries() -> Vec<(&'static str, Resolved)> {
    let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
    let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
    let function = |value| Resolved { kind: EntryKind::Function, value, recipes_agreeing: 2 };
    let mut entries = ClientGame::entries(&[]);
    entries.extend([
        ("ClientInstance.requestLeaveGame", slot(40)),
        ("MinecraftPackets.createPacket", function(0x1000)),
        ("ClientInstance.getPacketSender", slot(41)),
        ("PacketSender.send", slot(2)),
        ("CommandRequestPacket.mCommand", field(0x30)),
        ("CommandRequestPacket.size", field(0xA0)),
        ("MinecraftGame.getWorldSystem", slot(42)),
        ("WorldSystem.mLocalWorldStarter", field(0x78)),
        ("LocalWorldStarter.startLocalWorld", function(0x2000)),
    ]);
    entries
}

fn start() -> Harness {
    start_with(&entries())
}

fn start_with(entries: &[(&'static str, Resolved)]) -> Harness {
    let harness = Harness::start_with_entries(entries);
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    SessionActions::install(harness.state);
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

#[test]
fn the_offered_actions_follow_their_entries() {
    let harness = start();
    let areas = harness.state.areas();

    assert!(areas.is_available("client.leave_world", 1));
    assert!(areas.is_available("client.send_command", 1));
    assert!(areas.is_available("client.start_local_world", 1));
    assert!(!areas.is_available("client.join_server", 1));
    assert!(!areas.is_available("client.open_menu", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "client.open_menu");
    assert!(log.contains("area not available") && log.contains("OreUIRouter.pushRoute"), "{log}");
}

#[test]
fn the_world_actions_are_refused_on_the_title_screen_and_an_area_not_offered_is_not_available() {
    let harness = start();
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    let text = "/say hi";

    // SAFETY: the table is live; the views and `out` live for the calls.
    let (leave, command, menu) = unsafe {
        (
            table.leave_world.unwrap()(client),
            table.send_command.unwrap()(client, AmmlStr::from_str(text)),
            table.open_menu.unwrap()(client, amml_abi::AmmlMenu::SERVERS),
        )
    };

    assert_eq!((leave, command, menu), (AmmlStatus::REFUSED, AmmlStatus::REFUSED, AmmlStatus::NOT_AVAILABLE));
}

#[test]
fn a_world_start_is_refused_in_a_world_and_a_long_command_is_refused() {
    let harness = start();
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);
    let table = table(&harness);
    let client = handle(&GAME_CLIENT_A);
    let long = "x".repeat(SessionActions::MAX_COMMAND + 1);
    let mut outcome = AmmlStartWorldOutcome(0xEE);

    // SAFETY: the table is live; the views and `outcome` live for the calls.
    let (start, command) = unsafe {
        (
            table.start_local_world.unwrap()(client, AmmlStr::from_str("abc="), &mut outcome),
            table.send_command.unwrap()(client, AmmlStr::from_str(&long)),
        )
    };

    assert_eq!((start, command), (AmmlStatus::REFUSED, AmmlStatus::REFUSED));
    assert_eq!(outcome, AmmlStartWorldOutcome(0xEE));
}

// The session entries plus those of `client.join_server_direct`.
fn start_direct() -> Harness {
    start_with(&DirectJoinGame::with_entries(entries()))
}

#[test]
fn a_direct_join_calls_the_games_connect_by_address_with_the_address_and_the_port() {
    let harness = start_direct();
    assert!(harness.state.areas().is_available("client.join_server_direct", 1));
    let actions = SessionActions::get().unwrap();
    let client = handle(&GAME_CLIENT_A);

    let first = actions.join_server_direct(3, client, "127.0.0.1", 28231, "DejaVu replay").unwrap();

    // The game builds the connection info and the join context itself; the loader passes `this` (the subobject whose
    // vtable holds the function: the client itself here), the address and the port.
    assert_eq!(DirectJoinGame::connected(), [(client as usize, "127.0.0.1".to_string(), 28231)]);
    // The game's call has no result: the join is answered `OTHER` at once, so another join is not refused.
    assert_eq!(harness.state.owner_events().queued_for(3), 1);
    let second = actions.join_server_direct(3, client, "127.0.0.1", 28232, "").unwrap();
    assert!(second > first);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "direct server join asked");
    assert!(log.contains("\"this_offset\":\"0x0\"") && log.contains("\"r8d_port\":28231"), "{log}");
    assert!(log.contains("\"rdx_host\":\"127.0.0.1\""), "{log}");
}

#[test]
fn a_direct_join_is_refused_in_a_world_and_for_an_empty_host_or_port_zero() {
    let _harness = start_direct();
    let actions = SessionActions::get().unwrap();
    let client = handle(&GAME_CLIENT_A);

    let empty_host = actions.join_server_direct(3, client, "", 28231, "x");
    // The game builds the info again only for an address literal: a host name is refused.
    let host_name = actions.join_server_direct(3, client, "replay.example", 28231, "x");
    let leading_zero = actions.join_server_direct(3, client, "127.000.0.1", 28231, "x");
    let port_zero = actions.join_server_direct(3, client, "127.0.0.1", 0, "x");
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);
    let in_world = actions.join_server_direct(3, client, "127.0.0.1", 28231, "x");

    assert_eq!(
        (empty_host, host_name, leading_zero, port_zero, in_world),
        (
            Err(AmmlStatus::INVALID_ARGUMENT),
            Err(AmmlStatus::INVALID_ARGUMENT),
            Err(AmmlStatus::INVALID_ARGUMENT),
            Err(AmmlStatus::INVALID_ARGUMENT),
            Err(AmmlStatus::REFUSED)
        )
    );
    assert!(DirectJoinGame::connected().is_empty());
}

#[test]
fn a_direct_join_is_not_available_without_the_connect_function() {
    let entries: Vec<_> = DirectJoinGame::with_entries(entries())
        .into_iter()
        .filter(|(id, _)| *id != "ClientInstance.connectByAddress")
        .collect();
    let harness = start_with(&entries);
    let client = handle(&GAME_CLIENT_A);

    let status = SessionActions::get().unwrap().join_server_direct(3, client, "127.0.0.1", 28231, "x");

    assert_eq!(status, Err(AmmlStatus::NOT_AVAILABLE));
    assert!(!harness.state.areas().is_available("client.join_server_direct", 1));
    let table = table(&harness);
    assert_eq!(table.version, 8);
    assert!(table.join_server_direct.is_some());
}

#[test]
fn an_ipv6_direct_join_passes_the_address_in_its_canonical_form() {
    let _harness = start_direct();
    let client = handle(&GAME_CLIENT_A);

    SessionActions::get().unwrap().join_server_direct(3, client, "0:0:0:0:0:0:0:1", 28231, "").unwrap();

    assert_eq!(DirectJoinGame::connected()[0].1, "::1");
}

#[test]
fn a_direct_join_replaces_a_server_join_that_waits_and_answers_it() {
    let mut entries = WorldGame::entries(&[]);
    entries.extend(WorldGame::join_entries());
    let entries = DirectJoinGame::with_entries(entries);
    let harness = start_with(&entries);
    let actions = SessionActions::get().unwrap();
    let client = handle(&GAME_CLIENT_A);

    // The stand-in `joinExternalServer` never answers, as a lost join of the game.
    let waiting = actions.join_server(5, client, "127.0.0.1", 28231, "").unwrap();
    let second = actions.join_server(5, client, "127.0.0.1", 28231, "");
    let direct = actions.join_server_direct(3, client, "127.0.0.1", 28231, "").unwrap();

    assert_eq!(WorldGame::external_joins(), 1);
    assert_eq!(second, Err(AmmlStatus::REFUSED));
    assert!(direct > waiting);
    assert_eq!((harness.state.owner_events().queued_for(5), harness.state.owner_events().queued_for(3)), (1, 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "replaced by a direct join");
    assert!(log.contains(&format!("\"join\":{waiting}")), "{log}");
}

#[test]
fn a_direct_join_calls_connect_by_address_on_the_derived_subobject() {
    let mut entries = entries();
    entries.extend(DirectJoinGame::entries_at(0xA0));
    let harness = start_with(&entries);
    let client = handle(&GAME_CLIENT_A);

    SessionActions::get().unwrap().join_server_direct(3, client, "127.0.0.1", 28231, "").unwrap();

    // The 70.26 shape: the function is called through the base subobject at +0A0h, as the game's virtual call does.
    assert_eq!(DirectJoinGame::connected(), [(client as usize + 0xA0, "127.0.0.1".to_string(), 28231)]);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "direct server join asked");
    assert!(log.contains("\"this_offset\":\"0xa0\""), "{log}");
}

#[test]
fn a_direct_join_is_not_available_without_the_derived_subobject_offset() {
    let entries: Vec<_> = DirectJoinGame::with_entries(entries())
        .into_iter()
        .filter(|(id, _)| *id != "ClientInstance.connectByAddressThis")
        .collect();
    let harness = start_with(&entries);
    let client = handle(&GAME_CLIENT_A);

    let status = SessionActions::get().unwrap().join_server_direct(3, client, "127.0.0.1", 28231, "");

    assert_eq!(status, Err(AmmlStatus::NOT_AVAILABLE));
    assert!(!harness.state.areas().is_available("client.join_server_direct", 1));
    assert!(DirectJoinGame::connected().is_empty());
}
