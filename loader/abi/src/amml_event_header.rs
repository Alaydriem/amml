use crate::{AmmlOrigin, AmmlPhase};

/// The first field of every event struct.
///
/// `size` and `version` describe the whole event struct, not only the header. A listener that
/// cancels a cancellable event sets `cancelled` to a non-zero value.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlEventHeader {
    pub size: u32,
    pub version: u32,
    pub phase: AmmlPhase,
    pub origin: AmmlOrigin,
    pub cancelled: u8,
    pub injected: u8,
    pub _reserved: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlEventHeader>() == 16);
