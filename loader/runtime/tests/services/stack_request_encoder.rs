use amml_runtime::{RequestAction, SlotRef, StackRequestEncoder};

// The vanilla captures of the hotbar-manager findings (container ids report, section 3), without the `93 01` packet
// header: the encoder writes the payload and the loader makes the packet from it.
fn payload(hex: &str) -> Vec<u8> {
    let bytes: Vec<u8> = hex.split_whitespace().map(|byte| u8::from_str_radix(byte, 16).unwrap()).collect();
    assert_eq!(&bytes[..2], &[0x93, 0x01]);
    bytes[2..].to_vec()
}

fn slot(container: u8, slot: u8, net_id: i32) -> SlotRef {
    SlotRef { container, slot, net_id }
}

#[test]
fn swap_encodes_like_the_recorded_packet() {
    // #72: chest 7 (net 6) with hotbar 0 (net 13), request -35.
    let expected = payload("93 01 01 45 01 02 02 07 00 07 06 00 00 00 1c 00 00 0d 00 00 00 00 ff ff ff ff");

    let bytes = StackRequestEncoder::batch(&[(-35, RequestAction::Swap { from: slot(7, 7, 6), to: slot(28, 0, 13) })]);

    assert_eq!(bytes, expected);
}

#[test]
fn place_into_an_empty_slot_writes_the_net_id_zero() {
    // #70: chest 6 (2 gold, net 5) to empty hotbar 4, request -33. The caller names the target with its own net id;
    // the encoder writes 0 for a place into an empty slot.
    let expected = payload("93 01 01 41 01 01 01 02 07 00 06 05 00 00 00 1c 00 04 00 00 00 00 00 ff ff ff ff");

    let bytes = StackRequestEncoder::batch(&[(
        -33,
        RequestAction::Place { count: 2, from: slot(7, 6, 5), to: slot(28, 4, 99) },
    )]);

    assert_eq!(bytes, expected);
}

#[test]
fn place_onto_a_stack_keeps_the_net_id_of_the_target() {
    // #76: chest 8 (7 iron, net 7) onto hotbar 5 (30 iron, net 13), request -39.
    let expected = payload("93 01 01 4d 01 01 01 07 07 00 08 07 00 00 00 1c 00 05 0d 00 00 00 00 ff ff ff ff");

    let bytes = StackRequestEncoder::batch(&[(
        -39,
        RequestAction::PlaceOnto { count: 7, from: slot(7, 8, 7), to: slot(28, 5, 13) },
    )]);

    assert_eq!(bytes, expected);
}

#[test]
fn drop_encodes_like_the_recorded_packet() {
    // #63: the whole stack of main slot 9 (16 gold, net 15), request -27.
    let expected = payload("93 01 01 35 01 03 03 10 1d 00 09 0f 00 00 00 00 00 ff ff ff ff");

    let bytes = StackRequestEncoder::batch(&[(-27, RequestAction::Drop { count: 16, from: slot(29, 9, 15) })]);

    assert_eq!(bytes, expected);
}

#[test]
fn a_batch_writes_the_count_then_each_request() {
    let (a, b, c) = (slot(28, 0, 5), slot(29, 9, 7), slot(29, 18, 0));
    let first = StackRequestEncoder::batch(&[(-100001, RequestAction::Swap { from: a, to: b })]);
    let second = StackRequestEncoder::batch(&[(-100003, RequestAction::Place { count: 4, from: b, to: c })]);
    // Each single payload starts with its request count, 1, in one byte.
    let mut expected = vec![2u8];
    expected.extend_from_slice(&first[1..]);
    expected.extend_from_slice(&second[1..]);

    let bytes = StackRequestEncoder::batch(&[
        (-100001, RequestAction::Swap { from: a, to: b }),
        (-100003, RequestAction::Place { count: 4, from: b, to: c }),
    ]);

    assert_eq!(bytes, expected);
}

#[test]
fn a_request_id_of_two_bytes_is_a_varint() {
    // #201: main 10 (5 items, net 10) to empty hotbar 7, request -129.
    let expected = payload("93 01 01 81 02 01 01 01 05 1d 00 0a 0a 00 00 00 1c 00 07 00 00 00 00 00 ff ff ff ff");

    let bytes = StackRequestEncoder::batch(&[(
        -129,
        RequestAction::Place { count: 5, from: slot(29, 10, 10), to: slot(28, 7, 0) },
    )]);

    assert_eq!(bytes, expected);
}

// One request id as the encoder writes it: the zigzag varint of -100003 (C5 9A 0C) and the created stack's net id,
// which is the request's own id, as a little-endian signed int.
const ID_100003: [u8; 3] = [0xC5, 0x9A, 0x0C];

#[test]
fn craft_creative_action_encodes_the_build_values() {
    // The CraftCreative action is type 14 at variant index 12 on 26.63 and on 70.26 (survey section 7, I3); the bytes
    // are those of the old stratum encoder: the creative net id 300 (AC 02) as an unsigned varint and one craft, then
    // the take of the created stack (container 60, slot 50, the request id as net id) into the empty hotbar slot 4.
    let mut expected = vec![0x01];
    expected.extend_from_slice(&ID_100003);
    expected.extend_from_slice(&[0x02, 12, 14, 0xAC, 0x02, 0x01, 0x00, 0x00, 64, 60, 0x00, 50]);
    expected.extend_from_slice(&(-100003i32).to_le_bytes());
    expected.extend_from_slice(&[28, 0x00, 4, 0, 0, 0, 0]);
    expected.extend_from_slice(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF]);

    let bytes = StackRequestEncoder::batch(&[(
        -100003,
        RequestAction::CraftCreative { creative_net_id: 300, count: 64, to: slot(28, 4, 99), replacing: None },
    )]);

    assert_eq!(bytes, expected);
}

#[test]
fn pick_is_one_request() {
    let bytes = StackRequestEncoder::batch(&[(
        -100003,
        RequestAction::CraftCreative { creative_net_id: 300, count: 64, to: slot(28, 4, 0), replacing: None },
    )]);

    // One request in the packet, with the two actions of the creative craft.
    assert_eq!(bytes[0], 1);
    assert_eq!(&bytes[1..4], &ID_100003);
    assert_eq!(bytes[4], 2);
}

#[test]
fn a_pick_over_a_held_stack_destroys_it_first() {
    // The old stratum encoder's `pick_replacing`: Destroy (variant index 4, type 4) of the held 10 items in hotbar slot 4
    // with the held net id 77, then the craft and the take; the made stack goes into the slot, now empty (net id 0).
    let mut expected = vec![0x01];
    expected.extend_from_slice(&ID_100003);
    expected.extend_from_slice(&[0x03, 4, 4, 10, 28, 0x00, 4, 77, 0, 0, 0]);
    expected.extend_from_slice(&[12, 14, 0xAC, 0x02, 0x01, 0x00, 0x00, 64, 60, 0x00, 50]);
    expected.extend_from_slice(&(-100003i32).to_le_bytes());
    expected.extend_from_slice(&[28, 0x00, 4, 0, 0, 0, 0]);
    expected.extend_from_slice(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF]);

    let bytes = StackRequestEncoder::batch(&[(
        -100003,
        RequestAction::CraftCreative {
            creative_net_id: 300,
            count: 64,
            to: slot(28, 4, 0),
            replacing: Some((10, slot(28, 4, 77))),
        },
    )]);

    assert_eq!(bytes, expected);
}
