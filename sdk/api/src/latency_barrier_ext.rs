use amml_abi::LatencyBarrier;

/// Typed reads of a `LatencyBarrier`: the game handled a `NetworkStackLatencyPacket`, so every packet the server sent
/// before it is applied. Raised After the game's handler.
pub trait LatencyBarrierExt {
    /// The `createTime` field of the packet.
    fn create_time(&self) -> u64;
}

impl LatencyBarrierExt for LatencyBarrier {
    fn create_time(&self) -> u64 {
        self.create_time
    }
}
