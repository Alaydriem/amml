use std::ffi::c_void;

use amml_game::{EntryId, NetworkConnection};

use crate::support::{FakeEntries, FakeIdentifier, FakeNetwork, GameMemory};

#[test]
fn the_id_is_the_identifier_at_the_start() {
    let entries = FakeNetwork::entries();
    let id = FakeIdentifier::raknet(0x1122);
    let mut network = FakeNetwork::new(&[Some(&id)]);
    // SAFETY: a live fake connection.
    let connection = unsafe { NetworkConnection::from_raw(network.connection_ptr(0), &entries) }.unwrap();

    assert_eq!(connection.id().guid, 0x1122);
    assert_eq!(connection.id().kind, FakeIdentifier::RAKNET);
}

#[test]
fn the_peer_is_read_at_the_entry_offset_and_null_gives_none() {
    let entries = FakeNetwork::entries();
    let mut network = FakeNetwork::new(&[None]);
    let mut peer = GameMemory::new(16);

    network.connection(0).put::<u64>(FakeNetwork::PEER_AT, peer.ptr() as u64);
    // SAFETY: a live fake connection.
    let connection = unsafe { NetworkConnection::from_raw(network.connection_ptr(0), &entries) }.unwrap();
    assert_eq!(connection.peer(), Some(peer.ptr()));

    network.connection(0).put::<u64>(FakeNetwork::PEER_AT, 0);
    assert_eq!(connection.peer(), None);

    let none = FakeEntries::build(&[]);
    // SAFETY: no entry is present, so no field is read.
    let connection = unsafe { NetworkConnection::from_raw(network.connection_ptr(0), &none) }.unwrap();
    assert_eq!(connection.peer(), None);
}

#[test]
fn the_receive_slot_is_slot_eight_of_the_peer_vtable() {
    let entries = FakeNetwork::entries();
    let mut network = FakeNetwork::new(&[None]);
    let mut vtable = [0usize; 10];
    vtable[8] = 0xABCD;
    let mut peer = GameMemory::new(16);
    peer.put::<u64>(0, vtable.as_ptr() as u64);
    network.connection(0).put::<u64>(FakeNetwork::PEER_AT, peer.ptr() as u64);
    // SAFETY: a live fake connection.
    let connection = unsafe { NetworkConnection::from_raw(network.connection_ptr(0), &entries) }.unwrap();

    assert_eq!(connection.peer_receive_slot(), Some(0xABCD as *const c_void));

    // a null slot gives None; the vtable is read again on each call
    // SAFETY: `vtable` is a live local array; the volatile write keeps the compiler from assuming it is constant.
    unsafe { std::ptr::write_volatile(&raw mut vtable[8], 0) };
    assert_eq!(connection.peer_receive_slot(), None);
}

#[test]
fn the_receive_slot_is_none_when_the_entry_is_off_or_the_peer_is_null() {
    let mut network = FakeNetwork::new(&[None]);
    let off = FakeEntries::build(&[(EntryId::NETWORK_CONNECTION_PEER, FakeNetwork::PEER_AT as u64)]);
    let mut peer = GameMemory::new(16);
    network.connection(0).put::<u64>(FakeNetwork::PEER_AT, peer.ptr() as u64);
    // SAFETY: a live fake connection; the missing slot entry stops the read before the vtable is used.
    let connection = unsafe { NetworkConnection::from_raw(network.connection_ptr(0), &off) }.unwrap();
    assert_eq!(connection.peer_receive_slot(), None);

    let entries = FakeNetwork::entries();
    network.connection(0).put::<u64>(FakeNetwork::PEER_AT, 0);
    // SAFETY: a live fake connection.
    let connection = unsafe { NetworkConnection::from_raw(network.connection_ptr(0), &entries) }.unwrap();
    assert_eq!(connection.peer_receive_slot(), None);
}
