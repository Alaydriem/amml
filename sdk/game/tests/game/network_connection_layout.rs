use std::ffi::c_void;

use amml_game::NetworkConnectionLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_id_the_type_and_the_peer_at_the_game_offsets() {
    let mut memory = GameMemory::new(0x120);
    // mId +0 (its type at +0xA8), mType +0xB0, mPeer +0x118
    memory.put::<i32>(0xA8, 1);
    memory.put::<u64>(0x18, 77);
    memory.put::<i32>(0xB0, 4);
    memory.put::<usize>(0x118, 0x7FF6_0000_2000);

    // SAFETY: the buffer is 16-byte aligned, zeroed and covers the prefix.
    let connection = unsafe { &*memory.ptr().cast::<NetworkConnectionLayout>() };

    assert_eq!(connection.id.kind, 1);
    assert_eq!(connection.id.guid, 77);
    assert_eq!(connection.kind, 4);
    assert_eq!(connection.peer, 0x7FF6_0000_2000 as *mut c_void);
}
