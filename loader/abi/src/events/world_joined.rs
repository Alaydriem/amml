use crate::{AmmlEvent, AmmlEventHeader};

/// Runs when the client has joined a world; `epoch` names that world session.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WorldJoined {
    pub header: AmmlEventHeader,
    pub epoch: u64,
}

impl AmmlEvent for WorldJoined {
    const ID: &'static str = "world.joined";
}

const _: () = assert!(core::mem::size_of::<WorldJoined>() == 24);
