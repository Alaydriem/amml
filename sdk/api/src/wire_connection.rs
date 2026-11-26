use amml_abi::AmmlWireConnection;

use crate::WireKind;

/// The client connection of a `PacketWire` or `ConnectionStarted` event.
///
/// `id` is the loader's number for the connection: never 0, never given to another connection in the process; a world
/// leave forgets it. `covered` is false when the loader's receive hook does not see the connection's packets: such a
/// connection gives `Out` packets only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WireConnection {
    pub id: u64,
    pub kind: WireKind,
    pub covered: bool,
}

impl From<AmmlWireConnection> for WireConnection {
    /// `LOCAL` is `Local`; any other kind, which the loader never writes, reads as `Remote`.
    fn from(connection: AmmlWireConnection) -> Self {
        let kind = if connection.kind == AmmlWireConnection::LOCAL { WireKind::Local } else { WireKind::Remote };
        Self { id: connection.id, kind, covered: connection.covered != 0 }
    }
}

impl From<WireConnection> for AmmlWireConnection {
    fn from(connection: WireConnection) -> Self {
        let kind = match connection.kind {
            WireKind::Local => AmmlWireConnection::LOCAL,
            WireKind::Remote => AmmlWireConnection::REMOTE,
        };
        AmmlWireConnection { id: connection.id, kind, covered: u8::from(connection.covered), _reserved: [0; 6] }
    }
}
