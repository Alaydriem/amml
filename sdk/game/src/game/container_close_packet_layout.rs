use std::mem::offset_of;

use crate::game::PacketLayout;

/// Layout mirror of the game's `ContainerClosePacket` (56 bytes): the packet, the `ContainerClosePacketPayload`, then
/// `mSerializationMode`.
#[repr(C)]
pub struct ContainerClosePacketLayout {
    pub packet: PacketLayout,         // 0x00
    pub container_id: i8,             // 0x30 mContainerId (ContainerID)
    pub container_type: i8,           // 0x31 mContainerType (SharedTypes::Legacy::ContainerType: None -9)
    pub server_initiated_close: bool, // 0x32 mServerInitiatedClose
    _serialization_mode: i32,         // 0x34 mSerializationMode
}

const _: () = assert!(offset_of!(ContainerClosePacketLayout, container_id) == 0x30);
const _: () = assert!(offset_of!(ContainerClosePacketLayout, container_type) == 0x31);
const _: () = assert!(offset_of!(ContainerClosePacketLayout, server_initiated_close) == 0x32);
const _: () = assert!(offset_of!(ContainerClosePacketLayout, _serialization_mode) == 0x34);
const _: () = assert!(size_of::<ContainerClosePacketLayout>() == 0x38);
