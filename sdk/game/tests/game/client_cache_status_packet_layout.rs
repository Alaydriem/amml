use amml_game::ClientCacheStatusPacketLayout;

use crate::support::GameMemory;

#[test]
fn reads_and_writes_the_enabled_flag_at_0x30() {
    let mut memory = GameMemory::new(0x38);
    memory.put::<u8>(0x30, 1);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let packet = unsafe { &mut *memory.ptr().cast::<ClientCacheStatusPacketLayout>() };
    assert!(packet.enabled);

    packet.enabled = false;
    assert_eq!(memory.get::<u8>(0x30), 0);
}
