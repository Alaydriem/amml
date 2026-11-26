use amml_abi::AmmlWireConnection;
use amml_runtime::WireConnections;

const CONNECTION: u64 = 0x5000;
const PEER: u64 = 0x7000;

#[test]
fn new_id_starts_a_connection_once() {
    let mut connections = WireConnections::new();

    let (first, created) = connections.outbound(CONNECTION, PEER, WireConnections::REMOTE_TYPE, true);
    let (again, created_again) = connections.outbound(CONNECTION, PEER, WireConnections::REMOTE_TYPE, true);

    assert!(created);
    assert!(!created_again);
    assert_eq!(first, again);
    assert_ne!(first.id, 0);
    assert_eq!((first.kind, first.covered), (AmmlWireConnection::REMOTE, 1));
    // The start is pending until a listener got it, then given once.
    assert!(connections.start_pending(first.id));
    connections.mark_started(first.id);
    assert!(!connections.start_pending(first.id));
    // The receive side finds the connection by its peer.
    assert_eq!(connections.inbound(PEER), Some(first));
    assert_eq!(connections.inbound(PEER + 8), None);
    // Another connection, and a new peer at the same connection address, are new connections with new ids.
    let (local, local_created) =
        connections.outbound(CONNECTION + 0x200, PEER + 0x100, WireConnections::LOCAL_TYPE, true);
    assert!(local_created);
    assert_eq!(local.kind, AmmlWireConnection::LOCAL);
    assert_ne!(local.id, first.id);
    let (moved, moved_created) = connections.outbound(CONNECTION, PEER + 0x300, WireConnections::REMOTE_TYPE, true);
    assert!(moved_created);
    assert!(connections.start_pending(moved.id));
    assert_ne!(moved.id, first.id);
    assert_eq!(connections.inbound(PEER), None);
}

#[test]
fn ids_are_forgotten_at_world_left() {
    let mut connections = WireConnections::new();
    let (first, _) = connections.outbound(CONNECTION, PEER, WireConnections::REMOTE_TYPE, true);
    connections.mark_started(first.id);
    connections.note_system(0x9000, false);

    connections.forget();

    assert!(!connections.knows(CONNECTION, PEER));
    assert_eq!(connections.inbound(PEER), None);
    assert_eq!(connections.system(0x9000), None);
    // The same addresses after the forget start again, with an id never given before.
    let (again, created) = connections.outbound(CONNECTION, PEER, WireConnections::REMOTE_TYPE, true);
    assert!(created);
    assert!(connections.start_pending(again.id));
    assert_ne!(again.id, first.id);
}

#[test]
fn a_connection_gone_from_the_vector_is_forgotten() {
    let mut connections = WireConnections::new();
    let (old, _) = connections.outbound(CONNECTION, PEER, WireConnections::REMOTE_TYPE, true);
    let (kept, _) = connections.outbound(CONNECTION + 0x200, PEER + 0x100, WireConnections::REMOTE_TYPE, true);

    connections.keep_only(&[CONNECTION + 0x200]);

    // A peer address of the gone connection names nothing: a server peer that reuses it raises no client packet.
    assert_eq!(connections.inbound(PEER), None);
    assert_eq!(connections.inbound(PEER + 0x100), Some(kept));
    let (new, created) = connections.outbound(CONNECTION, PEER, WireConnections::REMOTE_TYPE, true);
    assert!(created);
    assert_ne!(new.id, old.id);
}

#[test]
fn uncovered_peer_is_marked() {
    let mut connections = WireConnections::new();

    let (connection, created) = connections.outbound(CONNECTION, PEER, WireConnections::REMOTE_TYPE, false);

    assert!(created);
    assert_eq!(connection.covered, 0);
    // The receive hook never sees an uncovered peer's packets; a call with it names no connection.
    assert_eq!(connections.inbound(PEER), None);
    let (_, created_again) = connections.outbound(CONNECTION, PEER, WireConnections::REMOTE_TYPE, false);
    assert!(!created_again);
}

#[test]
fn the_system_answer_is_kept_per_system() {
    let mut connections = WireConnections::new();
    assert_eq!(connections.system(0x9000), None);

    connections.note_system(0x9000, true);
    connections.note_system(0x9100, false);

    assert_eq!(connections.system(0x9000), Some(true));
    assert_eq!(connections.system(0x9100), Some(false));
}
