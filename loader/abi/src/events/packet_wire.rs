use crate::{AmmlEvent, AmmlEventHeader, AmmlWireConnection, AmmlWireDir};

/// One game packet of a client connection, as it crosses the wire: its header varint (packet id and sub-client ids)
/// first, then its body. This is the only event that lends raw game bytes.
///
/// `OUT`: raised Before `NetworkSystem::_sendInternal`, on the thread that sends (the context names it: `GAME` on the
/// game thread, else `NETWORK`); `bytes` is the serialized packet the game passes on to the connection's batch.
/// `IN`: raised After the outer peer's `receivePacket` gave one packet (after decryption, decompression and the batch
/// split), with `AmmlThread::NETWORK`. Only connections of the client network system are raised; the server system of a
/// local world raises nothing. Packets the loader gives to a game handler itself do not pass the wire and raise nothing.
///
/// `bytes` and `len` are valid only inside the listener call: the loader clears them after the dispatch, and the game
/// reuses the memory. A listener that needs the bytes later copies them in the call.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PacketWire {
    pub header: AmmlEventHeader,
    pub dir: AmmlWireDir,
    pub _reserved: [u8; 7],
    pub connection: AmmlWireConnection,
    pub bytes: *const u8,
    pub len: usize,
}

impl AmmlEvent for PacketWire {
    const ID: &'static str = "network.packet_wire";
}

const _: () = assert!(core::mem::size_of::<PacketWire>() == 56);
