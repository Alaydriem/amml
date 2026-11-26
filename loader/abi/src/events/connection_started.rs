use crate::{AmmlEvent, AmmlEventHeader, AmmlStr, AmmlWireConnection};

/// The loader saw the first packet of a client connection: raised once per connection, Before its first `PacketWire`
/// `OUT` event, on the thread that sends. A connection whose first packet went out while no mod listened is raised at
/// the first packet the loader sees for it.
///
/// `protocol` is the network protocol number of the game build (the derived `SharedConstants.NetworkProtocolVersion`).
/// For a `REMOTE` connection `host` and `port` are those of the client's game connection info when the game holds one
/// and the send runs on the game thread, else empty and 0; for a `LOCAL` connection they are always empty and 0. `host`
/// is valid only inside the listener call.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ConnectionStarted {
    pub header: AmmlEventHeader,
    pub connection: AmmlWireConnection,
    pub protocol: u32,
    pub port: u16,
    pub _reserved: u16,
    pub host: AmmlStr,
}

impl AmmlEvent for ConnectionStarted {
    const ID: &'static str = "network.connection_started";
}

const _: () = assert!(core::mem::size_of::<ConnectionStarted>() == 56);
