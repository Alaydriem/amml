use crate::PrimitiveShapeLayout;

/// The game entries the shapes call: `MinecraftPackets::createPacket`, the `PrimitiveShapesPacket` handler slot of the
/// client network handler and the offset of the shape vector in a `PrimitiveShapesPacket`.
#[derive(Clone, Copy, Debug)]
pub struct ShapeCalls {
    pub create_packet: usize,
    pub shapes_slot: usize,
    pub shapes_at: usize,
}

impl ShapeCalls {
    pub const HANDLER: &'static str = "ClientNetworkHandler.handle.PrimitiveShapesPacket";
    const CREATE: &'static str = "MinecraftPackets.createPacket";
    const SHAPES: &'static str = "PrimitiveShapesPacket.mShapes";
    const PAYLOAD: &'static str = "PrimitiveShapeDataPayload.size";

    /// Every call of the shapes, when each entry resolved and the payload size fits `PrimitiveShapeLayout`; else the
    /// entry that is missing or does not fit.
    pub fn from_lookup(lookup: &impl Fn(&str) -> Option<u64>) -> Result<Self, &'static str> {
        let value = |name: &'static str| lookup(name).and_then(|value| usize::try_from(value).ok()).ok_or(name);
        if lookup(Self::PAYLOAD).ok_or(Self::PAYLOAD)? != PrimitiveShapeLayout::SIZE {
            return Err(Self::PAYLOAD);
        }
        let create_packet = value(Self::CREATE)?;
        if create_packet == 0 {
            return Err(Self::CREATE);
        }
        Ok(Self { create_packet, shapes_slot: value(Self::HANDLER)?, shapes_at: value(Self::SHAPES)? })
    }
}
