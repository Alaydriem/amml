use amml_abi::PacketWire;

use crate::lent_wire::LentWire;
use crate::{WireConnection, WireDir};

/// Typed reads of a `PacketWire`: one game packet of a client connection, its header varint first, then its body.
///
/// `Out` packets are raised before the game sends them, on the sending thread; `In` packets after the outer peer gave
/// them (decrypted, decompressed and split from their batch), on the network thread. Only the client's connections
/// are raised, never the server system of a local world, and never a packet the loader gives to a game handler itself.
pub trait PacketWireExt {
    /// The direction of the packet.
    fn dir(&self) -> WireDir;

    /// The connection the packet belongs to.
    fn connection(&self) -> WireConnection;

    /// The packet's bytes, lent by the loader: valid only inside the listener call that got the event. A copy of the
    /// event, or one kept past its call, gives `None`. Copy the bytes in the call to keep them. This is the only event
    /// that lends raw game bytes.
    fn bytes(&self) -> Option<&[u8]>;
}

impl PacketWireExt for PacketWire {
    fn dir(&self) -> WireDir {
        WireDir::from(self.dir)
    }

    fn connection(&self) -> WireConnection {
        WireConnection::from(self.connection)
    }

    fn bytes(&self) -> Option<&[u8]> {
        LentWire::find(self)
    }
}
