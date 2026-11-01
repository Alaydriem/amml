use amml_game::NetworkIdentifierLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_type_and_the_compared_fields_at_the_game_offsets() {
    let mut memory = GameMemory::new(0xB0);
    // mNetherNetId variant: payload +0x00, index +0x10; mGuid.g +0x18; mSock +0x28 (port +0x2A, address +0x2C); mType +0xA8
    memory.put::<u64>(0x00, 0x1122_3344_5566_7788);
    memory.put::<u8>(0x10, 1);
    memory.put::<u64>(0x18, 0xAABB_CCDD_EEFF_0011);
    memory.put::<u16>(0x2A, 19132);
    memory.put::<u32>(0x2C, 0x0100_007F);
    memory.put::<i32>(0xA8, 3);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let id = unsafe { &*memory.ptr().cast::<NetworkIdentifierLayout>() };

    assert_eq!(id.kind, 3);
    assert_eq!(id.guid, 0xAABB_CCDD_EEFF_0011);
    assert_eq!(id.nether_net_id[0x10], 1);
    assert_eq!(u64::from_le_bytes(id.nether_net_id[..8].try_into().unwrap()), 0x1122_3344_5566_7788);
    assert_eq!(u16::from_le_bytes(id.sock[2..4].try_into().unwrap()), 19132);
    assert_eq!(u32::from_le_bytes(id.sock[4..8].try_into().unwrap()), 0x0100_007F);
}

#[test]
fn the_mirror_is_176_bytes_so_a_copy_takes_the_whole_id() {
    assert_eq!(size_of::<NetworkIdentifierLayout>(), 0xB0);
}
