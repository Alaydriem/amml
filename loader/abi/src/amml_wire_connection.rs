/// The client connection a `PacketWire` or `ConnectionStarted` event belongs to.
///
/// `id` is a loader number for the connection: never 0, never given to another connection in the same process. `kind`
/// is `LOCAL` (the client's connection to a world on this machine) or `REMOTE` (to another machine), from the game's
/// `NetworkConnection::mType`. `covered` is 1 when the loader's receive hook sees the packets of the connection (the
/// receive slot of its peer is the hooked function), else 0: an uncovered connection gives `OUT` packets only.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlWireConnection {
    pub id: u64,
    pub kind: u8,
    pub covered: u8,
    pub _reserved: [u8; 6],
}

impl AmmlWireConnection {
    /// `kind` of a connection to a world on this machine.
    pub const LOCAL: u8 = 1;
    /// `kind` of a connection to another machine.
    pub const REMOTE: u8 = 2;
}

const _: () = assert!(core::mem::size_of::<AmmlWireConnection>() == 16);
