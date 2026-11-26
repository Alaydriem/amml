use crate::{AmmlEvent, AmmlEventHeader};

/// Runs once per client tick.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ClientTick {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for ClientTick {
    const ID: &'static str = "client.tick";
}

const _: () = assert!(core::mem::size_of::<ClientTick>() == 16);
