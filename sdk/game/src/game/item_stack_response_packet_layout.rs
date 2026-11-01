use std::mem::offset_of;

use amml_msvc::MsvcVec;

use crate::game::{ItemStackResponseInfoLayout, PacketLayout};

/// Layout mirror of the game's `ItemStackResponsePacket` (80 bytes): the packet, the
/// `ItemStackResponsePacketPayload`, then `mSerializationMode`.
#[repr(C)]
pub struct ItemStackResponsePacketLayout {
    pub packet: PacketLayout,                            // 0x00
    pub responses: MsvcVec<ItemStackResponseInfoLayout>, // 0x30 mResponses
    _serialization_mode: i32,                            // 0x48 mSerializationMode
}

const _: () = assert!(offset_of!(ItemStackResponsePacketLayout, responses) == 0x30);
const _: () = assert!(offset_of!(ItemStackResponsePacketLayout, _serialization_mode) == 0x48);
const _: () = assert!(size_of::<ItemStackResponsePacketLayout>() == 0x50);
