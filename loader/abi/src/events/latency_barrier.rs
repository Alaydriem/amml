use crate::{AmmlEvent, AmmlEventHeader};

/// The game handled a `NetworkStackLatencyPacket`: raised After the game's handler, so every packet the server sent
/// before it is applied. Raised on the thread that runs the handler.
///
/// `create_time` is the `createTime` field of the packet.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LatencyBarrier {
    pub header: AmmlEventHeader,
    pub create_time: u64,
}

impl AmmlEvent for LatencyBarrier {
    const ID: &'static str = "time.latency_barrier";
}

const _: () = assert!(core::mem::size_of::<LatencyBarrier>() == 24);
