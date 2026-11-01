use std::ffi::c_void;

use amml_game::PacketLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_vtable_and_the_handled_flag_at_the_packet_offsets() {
    let mut memory = GameMemory::new(0x30);
    // Packet: vptr 0x00, mIsHandled 0x11
    memory.put::<usize>(0x00, 0x7FF6_0000_1000);
    memory.put::<u8>(0x11, 1);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let packet = unsafe { &*memory.ptr().cast::<PacketLayout>() };

    assert_eq!(packet.vtable, 0x7FF6_0000_1000 as *const c_void);
    assert!(packet.is_handled);
}
