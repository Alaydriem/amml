use crate::{AmmlInputBlocks, AmmlLeaseMode};

/// What a mod asks for in `AmmlInputTable::acquire`. `passthrough` lists `passthrough_count` keys
/// that real input keeps even while the lease blocks `KEYS`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlLeaseRequest {
    pub size: u32,
    pub mode: AmmlLeaseMode,
    pub preempt: u8,
    pub _reserved: [u8; 2],
    pub blocks: AmmlInputBlocks,
    pub _reserved2: u32,
    pub passthrough: *const u16,
    pub passthrough_count: usize,
}

impl AmmlLeaseRequest {
    /// `size` of a v1 lease request.
    pub const SIZE_V1: u32 = 32;
}

const _: () = assert!(core::mem::size_of::<AmmlLeaseRequest>() == AmmlLeaseRequest::SIZE_V1 as usize);
