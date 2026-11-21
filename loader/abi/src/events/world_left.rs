use crate::{AmmlEvent, AmmlEventHeader};

/// Runs when the client leaves a world; `epoch` names the session that ended.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WorldLeft {
    pub header: AmmlEventHeader,
    pub epoch: u64,
}

impl AmmlEvent for WorldLeft {
    const ID: &'static str = "world.left";
}

const _: () = assert!(core::mem::size_of::<WorldLeft>() == 24);
