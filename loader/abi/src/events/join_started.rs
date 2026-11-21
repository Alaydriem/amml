use crate::{AmmlEvent, AmmlEventHeader};

/// Runs when the client starts to join a world.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JoinStarted {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for JoinStarted {
    const ID: &'static str = "world.join_started";
}

const _: () = assert!(core::mem::size_of::<JoinStarted>() == 16);
