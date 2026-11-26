use std::ffi::c_void;
use std::sync::Mutex;

use amml::{ModListener, NetworkAdapter, SessionAdapter, WireAdapter, WireDetours, WorldInfoAdapter};
use amml_abi::{AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlStatus, AmmlThread, ConnectionStarted, PacketWire};

use crate::support::{
    CLIENT_A, ClientGame, GAME_CLIENT_A, Harness, InfoGame, LogFile, NetworkGame, SessionGame, WireGame,
};

const WIRE: &str = "network.packet_wire";
const STARTED: &str = "network.connection_started";
const PACKET: [u8; 6] = [0xc1, 0x01, 0x00, 0x00, 0x08, 0xb1];

// What the listeners saw, one line per call.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The addresses of the bytes the wire listeners were lent.
static LENT: Mutex<Vec<usize>> = Mutex::new(Vec::new());

fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&WireGame::entries_without(without));
    WireAdapter::install(harness.state);
    harness
}

// Listens to `event` as mod 9 in `band` with `callback`; `user` 1 makes a wire listener clear the bytes it got.
fn listen(
    harness: &Harness,
    event: &str,
    band: AmmlBand,
    callback: amml_abi::EventCallbackFn,
    user: usize,
) -> amml_runtime::ListenerId {
    let listener = ModListener { owner: 9, callback, user };
    harness.state.events().listen(event, 1, band, listener).unwrap()
}

fn listen_both(harness: &Harness) {
    listen(harness, STARTED, AmmlBand::NORMAL, record_started, 0);
    listen(harness, WIRE, AmmlBand::NORMAL, record_wire, 0);
}

unsafe extern "C" fn record_wire(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live `PacketWire` and context for the call.
    let (event, context) = unsafe { (&mut *event.cast::<PacketWire>(), &*context) };
    // SAFETY: the loader lends `len` bytes at `bytes` for the call.
    let bytes = unsafe { std::slice::from_raw_parts(event.bytes, event.len) };
    SEEN.lock().unwrap().push(format!(
        "wire dir={} id={} kind={} covered={} thread={} bytes={bytes:02x?}",
        event.dir.0, event.connection.id, event.connection.kind, event.connection.covered, context.thread.0
    ));
    LENT.lock().unwrap().push(event.bytes.expose_provenance());
    if !user.is_null() {
        event.bytes = std::ptr::null();
        event.len = 0;
    }
    AmmlStatus::OK
}

unsafe extern "C" fn record_started(
    event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    _user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live `ConnectionStarted` for the call, and its host view for the call.
    let event = unsafe { &*event.cast::<ConnectionStarted>() };
    // SAFETY: as above.
    let host = unsafe { std::slice::from_raw_parts(event.host.ptr, event.host.len) };
    SEEN.lock().unwrap().push(format!(
        "started id={} kind={} covered={} protocol={} port={} host={}",
        event.connection.id,
        event.connection.kind,
        event.connection.covered,
        event.protocol,
        event.port,
        String::from_utf8_lossy(host)
    ));
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

#[test]
fn a_client_connection_starts_then_raises_out_and_in() {
    let harness = install(&[]);
    listen_both(&harness);
    let (connection, peer) = WireGame::connection(3, WireGame::REMOTE, true);
    let system = WireGame::system(false, &[connection]);

    WireGame::send(system, WireGame::id(3), &PACKET);
    WireGame::receive(peer, &[0x8f, 0x01]);
    WireGame::send(system, WireGame::id(3), &[0x81, 0x01, 0x01]);

    let net = AmmlThread::NETWORK.0;
    assert_eq!(
        seen(),
        [
            "started id=1 kind=2 covered=1 protocol=2225 port=0 host=".to_string(),
            format!("wire dir=1 id=1 kind=2 covered=1 thread={net} bytes=[c1, 01, 00, 00, 08, b1]"),
            format!("wire dir=2 id=1 kind=2 covered=1 thread={net} bytes=[8f, 01]"),
            format!("wire dir=1 id=1 kind=2 covered=1 thread={net} bytes=[81, 01, 01]"),
        ]
    );
    assert_eq!((WireGame::sent(), WireGame::received()), (2, 1));
    // The system is asked once whether it is a server.
    assert_eq!(WireGame::is_server_calls(), 1);
}

#[test]
fn server_system_raises_nothing() {
    let harness = install(&[]);
    listen_both(&harness);
    let (connection, peer) = WireGame::connection(3, WireGame::LOCAL, true);
    let server = WireGame::system(true, &[connection]);

    WireGame::send(server, WireGame::id(3), &PACKET);
    WireGame::receive(peer, &[0x8f, 0x01]);

    assert!(seen().is_empty());
    // The game's own calls ran once each.
    assert_eq!((WireGame::sent(), WireGame::received()), (1, 1));
}

#[test]
fn bytes_are_lent_from_the_game_string_for_every_listener() {
    let harness = install(&[]);
    // The early listener clears the bytes it was lent; the later one still gets the game's bytes.
    listen(&harness, WIRE, AmmlBand::WRITE, record_wire, 1);
    listen(&harness, WIRE, AmmlBand::NORMAL, record_wire, 0);
    let (connection, _) = WireGame::connection(3, WireGame::REMOTE, true);
    let system = WireGame::system(false, &[connection]);

    let data = WireGame::send(system, WireGame::id(3), &PACKET);

    let lines = seen();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], lines[1]);
    // The bytes are lent from the game's string, not copied.
    assert_eq!(*LENT.lock().unwrap(), [data, data]);
    assert_eq!(WireGame::sent(), 1);
}

#[test]
fn uncovered_peer_is_marked() {
    let harness = install(&[]);
    listen_both(&harness);
    let (connection, _) = WireGame::connection(3, WireGame::LOCAL, false);
    let system = WireGame::system(false, &[connection]);

    WireGame::send(system, WireGame::id(3), &PACKET);

    let lines = seen();
    assert_eq!(lines[0], "started id=1 kind=1 covered=0 protocol=2225 port=0 host=");
    assert!(lines[1].starts_with("wire dir=1 id=1 kind=1 covered=0"));
}

#[test]
fn a_send_to_an_unknown_peer_raises_nothing() {
    let harness = install(&[]);
    listen_both(&harness);
    let (connection, _) = WireGame::connection(3, WireGame::REMOTE, true);
    let system = WireGame::system(false, &[connection]);

    WireGame::send(system, WireGame::id(4), &PACKET);

    assert!(seen().is_empty());
    assert_eq!(WireGame::sent(), 1);
}

#[test]
fn no_listener_reads_one_flags_word() {
    let harness = install(&[]);
    assert_eq!(WireDetours::flags(), 0);

    // A listener that came and went leaves the flags word clear again, so the detours raise nothing.
    let id = listen(&harness, WIRE, AmmlBand::NORMAL, record_wire, 0);
    assert_eq!(WireDetours::flags(), WireDetours::LISTENER);
    assert!(harness.state.events().unlisten(id));
    assert_eq!(WireDetours::flags(), 0);
    let (connection, peer) = WireGame::connection(3, WireGame::REMOTE, true);
    let system = WireGame::system(false, &[connection]);
    WireGame::send(system, WireGame::id(3), &PACKET);
    WireGame::receive(peer, &[0x8f]);

    assert_eq!((WireGame::sent(), WireGame::received()), (1, 1));
    // No game read ran: the system was never asked whether it is a server.
    assert_eq!(WireGame::is_server_calls(), 0);
    assert!(seen().is_empty());
}

#[test]
fn the_rows_are_not_offered_without_their_entries() {
    let harness = install(&["PacketTraceNetworkPeer.receivePacket"]);

    let events = harness.state.events();
    assert!(!events.is_available(WIRE, 1));
    assert!(!events.is_available(STARTED, 1));
}

#[test]
fn the_rows_are_offered_with_their_entries() {
    let harness = install(&[]);

    let events = harness.state.events();
    assert!(events.is_available(WIRE, 1));
    assert!(events.is_available(STARTED, 1));
}

#[test]
fn a_wire_listener_alone_does_not_use_up_the_start() {
    let harness = install(&[]);
    listen(&harness, WIRE, AmmlBand::NORMAL, record_wire, 0);
    let (connection, _) = WireGame::connection(3, WireGame::REMOTE, true);
    let system = WireGame::system(false, &[connection]);
    WireGame::send(system, WireGame::id(3), &[0x01]);

    // A `ConnectionStarted` listener that comes later gets the start at the next packet, once.
    listen(&harness, STARTED, AmmlBand::NORMAL, record_started, 0);
    WireGame::send(system, WireGame::id(3), &[0x02]);
    WireGame::send(system, WireGame::id(3), &[0x03]);

    let net = AmmlThread::NETWORK.0;
    assert_eq!(
        seen(),
        [
            format!("wire dir=1 id=1 kind=2 covered=1 thread={net} bytes=[01]"),
            "started id=1 kind=2 covered=1 protocol=2225 port=0 host=".to_string(),
            format!("wire dir=1 id=1 kind=2 covered=1 thread={net} bytes=[02]"),
            format!("wire dir=1 id=1 kind=2 covered=1 thread={net} bytes=[03]"),
        ]
    );
}

#[test]
fn a_failed_join_then_reused_addresses_start_again() {
    let mut entries = SessionGame::entries(&[]);
    entries.extend(WireGame::entries());
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    WireAdapter::install(harness.state);
    listen_both(&harness);
    let (connection, peer) = WireGame::connection(3, WireGame::REMOTE, true);
    let system = WireGame::system(false, &[connection]);
    WireGame::send(system, WireGame::id(3), &[0x01]);

    // The join ends before a world was joined: the exit forgets the connection, so a peer of the local server at the
    // freed address raises nothing, and a new connection at the same addresses starts again.
    SessionGame::exit();
    WireGame::receive(peer, &[0x8f]);
    WireGame::send(system, WireGame::id(3), &[0x02]);

    let starts: Vec<String> = seen().into_iter().filter(|line| line.starts_with("started")).collect();
    let ids: Vec<&str> = starts.iter().map(|line| line.split(' ').nth(1).unwrap()).collect();
    assert_eq!(ids, ["id=1", "id=2"]);
    assert!(seen().iter().all(|line| !line.starts_with("wire dir=2")));
}

#[test]
fn a_join_keeps_a_live_connection_and_its_in_packets() {
    let mut entries = SessionGame::entries(&[]);
    entries.extend(WireGame::entries());
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    WireAdapter::install(harness.state);
    listen_both(&harness);
    let (connection, peer) = WireGame::connection(3, WireGame::REMOTE, true);
    let system = WireGame::system(false, &[connection]);
    WireGame::send(system, WireGame::id(3), &[0x01]);

    // The game's join runs while the connection is live (packets may flow before `onStartJoinGame`): its IN packets
    // still raise, and it gets no second start.
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    WireGame::receive(peer, &[0x8f]);
    WireGame::send(system, WireGame::id(3), &[0x02]);

    let net = AmmlThread::NETWORK.0;
    assert_eq!(
        seen(),
        [
            "started id=1 kind=2 covered=1 protocol=2225 port=0 host=".to_string(),
            format!("wire dir=1 id=1 kind=2 covered=1 thread={net} bytes=[01]"),
            format!("wire dir=2 id=1 kind=2 covered=1 thread={net} bytes=[8f]"),
            format!("wire dir=1 id=1 kind=2 covered=1 thread={net} bytes=[02]"),
        ]
    );
}

#[test]
fn a_connection_gone_from_the_vector_is_forgotten() {
    let harness = install(&[]);
    listen_both(&harness);
    let (old, old_peer) = WireGame::connection(3, WireGame::LOCAL, true);
    WireGame::send(WireGame::system(false, &[old]), WireGame::id(3), &[0x01]);

    // A new connection of a system that no longer holds the old one: the old peer address names nothing after it.
    let (new, _) = WireGame::connection(4, WireGame::REMOTE, true);
    WireGame::send(WireGame::system(false, &[new]), WireGame::id(4), &[0x02]);
    WireGame::receive(old_peer, &[0x8f]);

    let lines = seen();
    assert_eq!(lines.len(), 4);
    assert!(lines[2].starts_with("started id=2 kind=2"));
    assert!(lines.iter().all(|line| !line.starts_with("wire dir=2")));
}

#[test]
fn a_remote_start_reads_the_host_and_port_on_the_game_thread() {
    let mut entries = ClientGame::entries(&[]);
    entries.extend(InfoGame::entries(InfoGame::SIZE_2663));
    entries.extend(WireGame::entries_without(&[
        "ClientInstance.getGameConnectionInfo",
        "GameConnectionInfo.size",
        "GameConnectionInfo.dtor",
    ]));
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    WorldInfoAdapter::install(harness.state);
    WireAdapter::install(harness.state);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    InfoGame::set_connection(Some((1, b"play.example.net", 19132)));
    listen_both(&harness);
    let (connection, _) = WireGame::connection(3, WireGame::REMOTE, true);

    WireGame::send(WireGame::system(false, &[connection]), WireGame::id(3), &PACKET);

    let game = AmmlThread::GAME.0;
    assert_eq!(
        seen(),
        [
            "started id=1 kind=2 covered=1 protocol=2225 port=19132 host=play.example.net".to_string(),
            format!("wire dir=1 id=1 kind=2 covered=1 thread={game} bytes=[c1, 01, 00, 00, 08, b1]"),
        ]
    );
}

#[test]
fn a_start_with_both_network_adapters_logs_no_missing_event_for_the_wire_rows() {
    let mut entries = WireGame::entries();
    let known: Vec<&str> = entries.iter().map(|(name, _)| *name).collect();
    entries.extend(NetworkGame::entries().into_iter().filter(|(name, _)| !known.contains(name)));
    let harness = Harness::start_with_entries(&entries);
    NetworkAdapter::install(harness.state);
    WireAdapter::install(harness.state);

    assert!(harness.state.events().is_available(WIRE, 1));
    assert!(harness.state.events().is_available(STARTED, 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "event offered");
    assert!(!log.contains("event not available"), "{log}");
}
