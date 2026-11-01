use std::mem::offset_of;

use crate::game::PacketLayout;

/// Layout mirror of the game's `NetworkStackLatencyPacket` (72 bytes): the packet, the payload, then
/// `mSerializationMode`.
#[repr(C)]
pub struct NetworkStackLatencyPacketLayout {
    pub packet: PacketLayout, // 0x00
    pub create_time: i64,     // 0x30 mCreateTime (steady_clock::time_point, raw count)
    pub from_server: bool,    // 0x38 mFromServer
    _payload_tail: [u8; 7],   // 0x39 padding to the 16-byte payload
    _serialization_mode: i32, // 0x40 mSerializationMode
}

const _: () = assert!(offset_of!(NetworkStackLatencyPacketLayout, create_time) == 0x30);
const _: () = assert!(offset_of!(NetworkStackLatencyPacketLayout, from_server) == 0x38);
const _: () = assert!(offset_of!(NetworkStackLatencyPacketLayout, _serialization_mode) == 0x40);
const _: () = assert!(size_of::<NetworkStackLatencyPacketLayout>() == 0x48);
