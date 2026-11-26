use std::mem::offset_of;

/// Layout mirror of the first 38h bytes of the game's `NetworkStackLatencyPacket` the loader reads, the same on 26.63
/// and 70.26 (survey section 8, row V1): the packet header, then `createTime` (8 bytes) at 30h.
///
/// The mirror is never built; the loader reads `createTime` in the live packet only.
#[repr(C)]
pub struct LatencyLayout {
    _header: [u8; 0x30],
    create_time: u64,
}

impl LatencyLayout {
    pub const CREATE_TIME_AT: usize = offset_of!(LatencyLayout, create_time);

    /// The `createTime` of the packet at `packet`.
    ///
    /// # Safety
    /// `packet` is the address of a live game `NetworkStackLatencyPacket`.
    pub unsafe fn create_time(packet: usize) -> u64 {
        let packet = std::ptr::with_exposed_provenance::<LatencyLayout>(packet);
        // SAFETY: the caller guarantees a live packet, which the mirror covers.
        unsafe { (&raw const (*packet).create_time).read() }
    }
}

const _: () = assert!(LatencyLayout::CREATE_TIME_AT == 0x30);
