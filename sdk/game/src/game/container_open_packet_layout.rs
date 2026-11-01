use std::mem::offset_of;

use crate::game::PacketLayout;

/// Layout mirror of the game's `ContainerOpenPacket` (80 bytes): the packet, the `ContainerOpenPacketPayload`, then
/// `mSerializationMode`.
#[repr(C)]
pub struct ContainerOpenPacketLayout {
    pub packet: PacketLayout, // 0x00
    pub container_id: i8,     // 0x30 mContainerId (ContainerID)
    pub container_type: i8,   // 0x31 mType (SharedTypes::Legacy::ContainerType: Inventory -1)
    _pos: [i32; 3],           // 0x34 mPos
    _entity_unique_id: i64,   // 0x40 mEntityUniqueID
    _serialization_mode: i32, // 0x48 mSerializationMode
}

const _: () = assert!(offset_of!(ContainerOpenPacketLayout, container_id) == 0x30);
const _: () = assert!(offset_of!(ContainerOpenPacketLayout, container_type) == 0x31);
const _: () = assert!(offset_of!(ContainerOpenPacketLayout, _serialization_mode) == 0x48);
const _: () = assert!(size_of::<ContainerOpenPacketLayout>() == 0x50);
