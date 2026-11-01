use amml_game::MovePlayerPacketLayout;

use crate::support::GameMemory;

#[test]
fn reads_every_field_at_the_handler_offsets() {
    let mut memory = GameMemory::new(0x80);
    memory.put::<u64>(0x30, 42); // mPlayerID
    memory.put::<[f32; 3]>(0x38, [1.0, 2.0, 3.0]); // mPos
    memory.put::<[f32; 2]>(0x44, [10.0, 20.0]); // mRot
    memory.put::<f32>(0x4C, 30.0); // mYHeadRot
    memory.put::<u8>(0x50, 2); // mResetPosition
    memory.put::<u8>(0x51, 1); // mOnGround
    memory.put::<u64>(0x58, 99); // mRidingID
    memory.put::<[i32; 2]>(0x60, [5, 6]); // mTeleportData
    memory.put::<u8>(0x68, 1); // engaged
    memory.put::<u64>(0x70, 1234); // mTick

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let packet = unsafe { &*memory.ptr().cast::<MovePlayerPacketLayout>() };

    assert_eq!(packet.player_id, 42);
    assert_eq!(packet.pos, [1.0, 2.0, 3.0]);
    assert_eq!(packet.rot, [10.0, 20.0]);
    assert_eq!(packet.y_head_rot, 30.0);
    assert_eq!(packet.reset_position, 2);
    assert!(packet.on_ground);
    assert_eq!(packet.riding_id, 99);
    assert_eq!(packet.teleport_data.get(), Some(&[5, 6]));
    assert_eq!(packet.tick, 1234);
}

#[test]
fn a_clear_engaged_flag_means_no_teleport_data() {
    let mut memory = GameMemory::new(0x80);
    memory.put::<[i32; 2]>(0x60, [5, 6]);

    // SAFETY: as above.
    let packet = unsafe { &*memory.ptr().cast::<MovePlayerPacketLayout>() };

    assert!(packet.teleport_data.get().is_none());
}
