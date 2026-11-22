use amml_abi::ConnectionStarted;

use crate::WireConnection;
use crate::lent_wire::LentWire;

/// Typed reads of a `ConnectionStarted`: the loader saw the first packet of a client connection, before its first
/// `PacketWire` `Out` event.
pub trait ConnectionStartedExt {
    /// The connection that started.
    fn connection(&self) -> WireConnection;

    /// The network protocol number of the game build.
    fn protocol(&self) -> u32;

    /// The port of a remote connection, from the client's connection info; 0 when the loader could not read it, and
    /// always 0 for a local connection.
    fn port(&self) -> u16;

    /// The host of a remote connection, from the client's connection info. Valid only inside the listener call: a copy
    /// of the event, or one kept past its call, gives `None`, as do an empty host, a local connection and a text that
    /// is not UTF-8.
    fn host(&self) -> Option<&str>;
}

impl ConnectionStartedExt for ConnectionStarted {
    fn connection(&self) -> WireConnection {
        WireConnection::from(self.connection)
    }

    fn protocol(&self) -> u32 {
        self.protocol
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn host(&self) -> Option<&str> {
        LentWire::find(self).filter(|bytes| !bytes.is_empty()).and_then(|bytes| std::str::from_utf8(bytes).ok())
    }
}
