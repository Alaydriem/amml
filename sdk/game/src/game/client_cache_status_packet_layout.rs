use std::mem::offset_of;

use crate::game::PacketLayout;

/// Layout mirror of the game's `ClientCacheStatusPacket` (56 bytes): the packet, the payload, then
/// `mSerializationMode`.
#[repr(C)]
pub struct ClientCacheStatusPacketLayout {
    pub packet: PacketLayout, // 0x00
    pub enabled: bool,        // 0x30 mEnabled
    _serialization_mode: i32, // 0x34 mSerializationMode
}

const _: () = assert!(offset_of!(ClientCacheStatusPacketLayout, enabled) == 0x30);
const _: () = assert!(offset_of!(ClientCacheStatusPacketLayout, _serialization_mode) == 0x34);
const _: () = assert!(size_of::<ClientCacheStatusPacketLayout>() == 0x38);
