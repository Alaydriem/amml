use crate::{AmmlEvent, AmmlEventHeader, AmmlLostReason};

/// An input lease was lost.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InputLeaseLost {
    pub header: AmmlEventHeader,
    pub lease: u64,
    pub reason: AmmlLostReason,
}

impl AmmlEvent for InputLeaseLost {
    const ID: &'static str = "input.lease_lost";
}

const _: () = assert!(core::mem::size_of::<InputLeaseLost>() == 32);
