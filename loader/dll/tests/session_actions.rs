use std::ffi::c_void;

use amml::{ClientAdapter, SessionActions, SessionAdapter};
use amml_abi::{AmmlApi, AmmlClientTable, AmmlServices, AmmlStartWorldOutcome, AmmlStatus, AmmlStr};
use amml_data::{EntryKind, Resolved};

use crate::support::{ClientGame, FakeGameClient, GAME_CLIENT_A, Harness, LogFile};

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
    let harness = Harness::start_with_entries(&entries());
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
