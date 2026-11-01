use amml_game::ContainerClosePacketLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_container_id_after_the_packet() {
    let mut memory = GameMemory::new(0x38);
    // PayloadPacket<ContainerClosePacketPayload>: mContainerId 0x30
    memory.put::<i8>(0x30, 9);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let packet = unsafe { &*memory.ptr().cast::<ContainerClosePacketLayout>() };

    assert_eq!(packet.container_id, 9);
}

#[test]
fn writes_the_id_type_and_server_flag_at_the_payload_offsets() {
    let mut memory = GameMemory::new(0x38);
    memory.put::<u8>(0x32, 1);

    // SAFETY: as above; the mirror is the only view of the buffer while it lives.
    let packet = unsafe { &mut *memory.ptr().cast::<ContainerClosePacketLayout>() };
    packet.container_id = 2;
    packet.container_type = -9;
    packet.server_initiated_close = false;

    // mContainerId 0x30, mContainerType 0x31, mServerInitiatedClose 0x32
    assert_eq!(memory.get::<i8>(0x30), 2);
    assert_eq!(memory.get::<i8>(0x31), -9);
    assert_eq!(memory.get::<u8>(0x32), 0);
}
