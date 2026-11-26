use crate::{AmmlEvent, AmmlEventHeader};

/// A suspended input lease is active again.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InputLeaseResumed {
    pub header: AmmlEventHeader,
    pub lease: u64,
}

impl AmmlEvent for InputLeaseResumed {
    const ID: &'static str = "input.lease_resumed";
}

const _: () = assert!(core::mem::size_of::<InputLeaseResumed>() == 24);
