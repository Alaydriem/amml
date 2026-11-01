use amml_game::ContainerOpenPacketLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_container_id_and_type_after_the_packet() {
    let mut memory = GameMemory::new(0x50);
    // PayloadPacket<ContainerOpenPacketPayload>: mContainerId 0x30, mType 0x31
    memory.put::<i8>(0x30, 4);
    memory.put::<i8>(0x31, -1);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let packet = unsafe { &*memory.ptr().cast::<ContainerOpenPacketLayout>() };

    assert_eq!(packet.container_id, 4);
    assert_eq!(packet.container_type, -1);
}
