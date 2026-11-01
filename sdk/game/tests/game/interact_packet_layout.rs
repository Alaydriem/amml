use amml_game::InteractPacketLayout;

use crate::support::GameMemory;

#[test]
fn writes_the_action_and_target_at_the_payload_offsets() {
    let mut memory = GameMemory::new(0x58);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror; the mirror is its only view.
    let packet = unsafe { &mut *memory.ptr().cast::<InteractPacketLayout>() };
    packet.action = 6;
    packet.target_id = 0x0000_0001_0000_0042;

    // PayloadPacket<InteractPacketPayload>: mAction 0x30, mTargetId 0x38
    assert_eq!(memory.get::<u8>(0x30), 6);
    assert_eq!(memory.get::<u64>(0x38), 0x0000_0001_0000_0042);
}
