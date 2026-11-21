use crate::LoaderState;

/// The derived values `RequestAdapter` makes and sends packets with: the packet factory, the slot of the packet sender
/// in the client vtable, the slots of `PacketSender.send` and `Packet.read`, the read-only stream the game reads a
/// packet from, the runtime id read, the slot swap of the player container and the offset of the net id in a stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestCalls {
    pub create_packet: usize,
    pub sender_slot: usize,
    pub send_slot: usize,
    pub read_slot: usize,
    pub stream_vtable: usize,
    pub stream_dtor: usize,
    pub get_runtime_id: usize,
    pub swap_slots: usize,
    pub net_id_at: usize,
}

impl RequestCalls {
    /// The calls from the resolved entries; `None` when one is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        Some(Self {
            create_packet: value("MinecraftPackets.createPacket").filter(|value| *value != 0)?,
            sender_slot: value("ClientInstance.getPacketSender")?,
            send_slot: value("PacketSender.send")?,
            read_slot: value("Packet.read")?,
            stream_vtable: value("ReadOnlyBinaryStream.vtable").filter(|value| *value != 0)?,
            stream_dtor: value("ReadOnlyBinaryStream.dtor").filter(|value| *value != 0)?,
            get_runtime_id: value("Actor.getRuntimeID").filter(|value| *value != 0)?,
            swap_slots: value("FillingContainer.swapSlots")?,
            // An offset of 0 would read the stack's item pointer as the net id.
            net_id_at: value("ItemStack.mNetIdVariant").filter(|value| *value != 0)?,
        })
    }
}
