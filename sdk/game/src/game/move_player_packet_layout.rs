use std::mem::offset_of;

use amml_msvc::Optional;

use crate::game::PacketLayout;

/// Layout mirror of the game's `MovePlayerPacket` (128 bytes): the packet, the payload, then `mSerializationMode`.
#[repr(C)]
pub struct MovePlayerPacketLayout {
    pub packet: PacketLayout,              // 0x00
    pub player_id: u64,                    // 0x30 mPlayerID (ActorRuntimeID)
    pub pos: [f32; 3],                     // 0x38 mPos
    pub rot: [f32; 2],                     // 0x44 mRot (x pitch, y yaw)
    pub y_head_rot: f32,                   // 0x4C mYHeadRot
    pub reset_position: u8,                // 0x50 mResetPosition
    pub on_ground: bool,                   // 0x51 mOnGround
    pub riding_id: u64,                    // 0x58 mRidingID
    pub teleport_data: Optional<[i32; 2]>, // 0x60 mTeleportData (cause, source entity type), engaged flag +0x68
    pub tick: u64,                         // 0x70 mTick
    _serialization_mode: i32,              // 0x78 mSerializationMode
}

const _: () = assert!(offset_of!(MovePlayerPacketLayout, player_id) == 0x30);
const _: () = assert!(offset_of!(MovePlayerPacketLayout, pos) == 0x38);
const _: () = assert!(offset_of!(MovePlayerPacketLayout, rot) == 0x44);
const _: () = assert!(offset_of!(MovePlayerPacketLayout, y_head_rot) == 0x4C);
const _: () = assert!(offset_of!(MovePlayerPacketLayout, reset_position) == 0x50);
const _: () = assert!(offset_of!(MovePlayerPacketLayout, on_ground) == 0x51);
const _: () = assert!(offset_of!(MovePlayerPacketLayout, riding_id) == 0x58);
const _: () = assert!(offset_of!(MovePlayerPacketLayout, teleport_data) == 0x60);
const _: () = assert!(offset_of!(MovePlayerPacketLayout, tick) == 0x70);
const _: () = assert!(offset_of!(MovePlayerPacketLayout, _serialization_mode) == 0x78);
const _: () = assert!(size_of::<MovePlayerPacketLayout>() == 0x80);
