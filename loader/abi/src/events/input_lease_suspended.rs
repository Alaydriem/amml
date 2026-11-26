use crate::{AmmlEvent, AmmlEventHeader};

/// An input lease was suspended by a preempting lease.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InputLeaseSuspended {
    pub header: AmmlEventHeader,
    pub lease: u64,
}

impl AmmlEvent for InputLeaseSuspended {
    const ID: &'static str = "input.lease_suspended";
}

const _: () = assert!(core::mem::size_of::<InputLeaseSuspended>() == 24);
