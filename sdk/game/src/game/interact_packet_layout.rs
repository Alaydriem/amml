use std::mem::offset_of;

use amml_msvc::Optional;

use crate::game::PacketLayout;

/// Layout mirror of the game's `InteractPacket` (88 bytes): the packet, the `InteractPacketPayload`, then
/// `mSerializationMode`.
#[repr(C)]
pub struct InteractPacketLayout {
    pub packet: PacketLayout, // 0x00
    pub action: u8,           // 0x30 mAction (InteractPacketPayload::Action: OpenInventory 6)
    pub target_id: u64,       // 0x38 mTargetId (ActorRuntimeID)
    _pos: Optional<[f32; 3]>, // 0x40 mPos
    _serialization_mode: i32, // 0x50 mSerializationMode
}

const _: () = assert!(offset_of!(InteractPacketLayout, action) == 0x30);
const _: () = assert!(offset_of!(InteractPacketLayout, target_id) == 0x38);
const _: () = assert!(offset_of!(InteractPacketLayout, _pos) == 0x40);
const _: () = assert!(offset_of!(InteractPacketLayout, _serialization_mode) == 0x50);
const _: () = assert!(size_of::<InteractPacketLayout>() == 0x58);
