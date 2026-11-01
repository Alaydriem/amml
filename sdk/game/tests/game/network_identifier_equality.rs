use amml_game::NetworkIdentifierEquality;

use crate::support::FakeIdentifier;

const A6: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

#[test]
fn ids_of_different_types_are_never_equal() {
    let equality = NetworkIdentifierEquality::new();
    let a = FakeIdentifier::raknet(0);
    let b = FakeIdentifier::blank(FakeIdentifier::ADDRESS);

    assert!(!equality.equals(&a, &b));
    assert!(!equality.equals(&b, &a));
}

#[test]
fn raknet_compares_only_the_guid() {
    let equality = NetworkIdentifierEquality::new();
    let mut a = FakeIdentifier::raknet(77);
    let mut b = FakeIdentifier::raknet(77);
    // a field the game does not compare for RakNet
    a.sock[2] = 9;
    b.nether_net_id[0] = 5;
    assert!(equality.equals(&a, &b));

    assert!(!equality.equals(&a, &FakeIdentifier::raknet(78)));
}

#[test]
fn address_compares_the_port_and_the_ipv4_address_only() {
    let equality = NetworkIdentifierEquality::new();
    let a = FakeIdentifier::address(19132, [10, 0, 0, 1]);
    let mut same = FakeIdentifier::address(19132, [10, 0, 0, 1]);
    // fields outside port and address, which the game does not compare
    same.sock[0] = 2;
    same.sock[8] = 0xFF;
    same.guid = 5;
    assert!(equality.equals(&a, &same));

    assert!(!equality.equals(&a, &FakeIdentifier::address(19133, [10, 0, 0, 1])));
    assert!(!equality.equals(&a, &FakeIdentifier::address(19132, [10, 0, 0, 2])));
}

#[test]
fn address6_compares_the_port_and_all_sixteen_address_bytes() {
    let equality = NetworkIdentifierEquality::new();
    let a = FakeIdentifier::address6(19133, A6);
    let mut same = FakeIdentifier::address6(19133, A6);
    // the sockaddr_in6 flow info at +4 is not compared
    same.sock[4] = 0xAA;
    assert!(equality.equals(&a, &same));

    assert!(!equality.equals(&a, &FakeIdentifier::address6(19134, A6)));
    for byte in [0, 15] {
        let mut other = A6;
        other[byte] ^= 1;
        assert!(!equality.equals(&a, &FakeIdentifier::address6(19133, other)), "byte {byte}");
    }
}

#[test]
fn nether_net_compares_the_index_then_the_bytes_the_index_names() {
    let equality = NetworkIdentifierEquality::new();
    let mut payload = [0u8; 16];
    payload[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);

    // index 1 compares 8 bytes
    let a = FakeIdentifier::nether_net(1, payload);
    let mut tail_differs = payload;
    tail_differs[12] = 0xEE;
    assert!(equality.equals(&a, &FakeIdentifier::nether_net(1, tail_differs)));
    let mut head_differs = payload;
    head_differs[7] = 0xEE;
    assert!(!equality.equals(&a, &FakeIdentifier::nether_net(1, head_differs)));

    // index 2 compares 16 bytes
    let b = FakeIdentifier::nether_net(2, payload);
    assert!(!equality.equals(&b, &FakeIdentifier::nether_net(2, tail_differs)));
    assert!(equality.equals(&b, &FakeIdentifier::nether_net(2, payload)));

    // index 0 is equal whatever the bytes are
    assert!(equality.equals(&FakeIdentifier::nether_net(0, payload), &FakeIdentifier::nether_net(0, head_differs)));

    // different indices are unequal even with equal bytes
    assert!(!equality.equals(&a, &b));
}

#[test]
fn invalid_ids_are_equal_to_each_other() {
    let equality = NetworkIdentifierEquality::new();
    let mut a = FakeIdentifier::blank(FakeIdentifier::INVALID);
    a.guid = 1;

    assert!(equality.equals(&a, &FakeIdentifier::blank(FakeIdentifier::INVALID)));
}

#[test]
fn an_unknown_type_is_never_equal() {
    let equality = NetworkIdentifierEquality::new();
    let a = FakeIdentifier::blank(9);

    assert!(!equality.equals(&a, &FakeIdentifier::blank(9)));
}

#[test]
fn nether_net_indices_above_two_are_equal_whatever_the_bytes_are() {
    let equality = NetworkIdentifierEquality::new();
    let (x, y) = ([1u8; 16], [2u8; 16]);

    for index in [3, 4, 0xFF] {
        assert!(equality.equals(&FakeIdentifier::nether_net(index, x), &FakeIdentifier::nether_net(index, y)));
    }
    assert!(!equality.equals(&FakeIdentifier::nether_net(3, x), &FakeIdentifier::nether_net(0xFF, x)));
}
