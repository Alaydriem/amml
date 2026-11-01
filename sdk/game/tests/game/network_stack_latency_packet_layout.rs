use amml_game::NetworkStackLatencyPacketLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_create_time_and_the_direction_flag_after_the_packet() {
    let mut memory = GameMemory::new(0x48);
    memory.put::<i64>(0x30, 123_456_789); // mCreateTime
    memory.put::<u8>(0x38, 1); // mFromServer

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let packet = unsafe { &*memory.ptr().cast::<NetworkStackLatencyPacketLayout>() };

    assert_eq!(packet.create_time, 123_456_789);
    assert!(packet.from_server);
}
