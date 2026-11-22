/// The direction of one game packet of a `PacketWire` event. A plain integer so that a value from the other side of
/// the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlWireDir(pub u8);

impl AmmlWireDir {
    /// A packet the client sends (serverbound).
    pub const OUT: Self = Self(1);
    /// A packet the client receives (clientbound).
    pub const IN: Self = Self(2);
}
