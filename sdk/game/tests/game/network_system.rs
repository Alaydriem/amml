use std::ffi::c_void;

use amml_game::{EntryId, GameEntries, NetworkSystem};

use crate::support::{FakeEntries, FakeIdentifier, FakeNetwork};

fn system<'a>(network: &mut FakeNetwork, entries: &'a GameEntries) -> NetworkSystem<'a> {
    // SAFETY: a live fake system with the vector at the entry's offset.
    unsafe { NetworkSystem::from_raw(network.system_ptr(), entries) }.unwrap()
}

#[test]
fn an_empty_vector_finds_nothing() {
    let entries = FakeNetwork::entries();
    let mut network = FakeNetwork::new(&[]);
    let wanted = FakeIdentifier::raknet(1);

    assert!(system(&mut network, &entries).connection_for(&wanted).is_none());
}

#[test]
fn a_cleared_vector_with_equal_non_null_begin_and_end_finds_nothing() {
    let entries = FakeNetwork::entries();
    let mut network = FakeNetwork::cleared();

    assert!(system(&mut network, &entries).connection_for(&FakeIdentifier::raknet(1)).is_none());
}

#[test]
fn a_null_element_is_skipped_and_never_read() {
    let entries = FakeNetwork::entries();
    let first = FakeIdentifier::raknet(1);
    let mut network = FakeNetwork::new(&[None, Some(&first)]);
    let expected = network.connection_ptr(1);

    let found = system(&mut network, &entries).connection_for(&first);

    assert_eq!(found.map(|connection| connection.as_ptr()), Some(expected));
}

#[test]
fn a_match_at_index_two_gives_that_connection() {
    let entries = FakeNetwork::entries();
    let ids = [FakeIdentifier::raknet(1), FakeIdentifier::address(19132, [1, 2, 3, 4]), FakeIdentifier::raknet(3)];
    let mut network = FakeNetwork::new(&[Some(&ids[0]), Some(&ids[1]), Some(&ids[2])]);
    let expected = network.connection_ptr(2);

    let found = system(&mut network, &entries).connection_for(&FakeIdentifier::raknet(3));

    assert_eq!(found.map(|connection| connection.as_ptr()), Some(expected));
}

#[test]
fn the_first_of_two_equal_connections_wins() {
    let entries = FakeNetwork::entries();
    let ids = [FakeIdentifier::raknet(5), FakeIdentifier::raknet(5)];
    let mut network = FakeNetwork::new(&[Some(&ids[0]), Some(&ids[1])]);
    let expected = network.connection_ptr(0);

    let found = system(&mut network, &entries).connection_for(&FakeIdentifier::raknet(5));

    assert_eq!(found.map(|connection| connection.as_ptr()), Some(expected));
}

#[test]
fn an_id_no_connection_has_gives_none() {
    let entries = FakeNetwork::entries();
    let ids = [FakeIdentifier::raknet(1), FakeIdentifier::raknet(2)];
    let mut network = FakeNetwork::new(&[Some(&ids[0]), None, Some(&ids[1])]);

    let system = system(&mut network, &entries);

    assert!(system.connection_for(&FakeIdentifier::raknet(3)).is_none());
    assert!(system.connection_for(&FakeIdentifier::address(1, [1, 1, 1, 1])).is_none());
}

#[test]
fn the_entry_being_off_gives_none() {
    let none = FakeEntries::build(&[]);
    let first = FakeIdentifier::raknet(1);
    let mut network = FakeNetwork::new(&[Some(&first)]);

    assert!(system(&mut network, &none).connection_for(&first).is_none());
    assert!(system(&mut network, &none).is_server().is_none());
}

extern "C" fn is_server_true(_: *mut c_void) -> u8 {
    1
}

extern "C" fn is_server_false(_: *mut c_void) -> u8 {
    0
}

#[test]
fn is_server_calls_the_function_the_entry_names() {
    let functions: [(extern "C" fn(*mut c_void) -> u8, bool); 2] = [(is_server_true, true), (is_server_false, false)];
    for (function, expected) in functions {
        let entries = FakeEntries::build(&[(EntryId::NETWORK_SYSTEM_IS_SERVER, function as usize as u64)]);
        let mut network = FakeNetwork::new(&[]);

        assert_eq!(system(&mut network, &entries).is_server(), Some(expected));
    }
}

#[test]
fn a_null_system_pointer_gives_none() {
    let entries = FakeNetwork::entries();

    // SAFETY: null is allowed.
    assert!(unsafe { NetworkSystem::from_raw(std::ptr::null_mut(), &entries) }.is_none());
}
