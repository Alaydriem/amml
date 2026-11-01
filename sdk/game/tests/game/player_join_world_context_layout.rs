use amml_game::PlayerJoinWorldContextLayout;

use crate::support::GameMemory;

#[test]
fn the_default_value_is_zero_except_the_small_string_capacity_at_0x18() {
    let context = PlayerJoinWorldContextLayout::default_value();

    // SAFETY: the mirror is `repr(C)`, 0xB0 bytes, and every byte of it is initialised.
    let bytes = unsafe { std::slice::from_raw_parts((&raw const context).cast::<u8>(), 0xB0) };

    for (offset, byte) in bytes.iter().enumerate() {
        let expected = if offset == 0x18 { 0x0F } else { 0 };
        assert_eq!(*byte, expected, "byte {offset:#x}");
    }
}

#[test]
fn reads_the_flags_at_the_game_offsets() {
    let mut memory = GameMemory::new(0xB0);
    memory.put::<u8>(0x20, 1); // isPartyLeader
    memory.put::<u8>(0x22, 1); // isServerTransfer
    memory.put::<u8>(0x23, 2); // reconnectionType

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let context = unsafe { &*memory.ptr().cast::<PlayerJoinWorldContextLayout>() };

    assert!(context.is_party_leader);
    assert!(!context.is_party_destination);
    assert!(context.is_server_transfer);
    assert_eq!(context.reconnection_type, 2);
}
