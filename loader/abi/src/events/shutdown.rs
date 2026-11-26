use crate::{AmmlEvent, AmmlEventHeader};

/// Runs once when the loader shuts down.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Shutdown {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for Shutdown {
    const ID: &'static str = "loader.shutdown";
}

const _: () = assert!(core::mem::size_of::<Shutdown>() == 16);
