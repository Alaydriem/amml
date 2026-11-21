use crate::{AmmlEvent, AmmlEventHeader};

/// Runs once after every mod has loaded.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModsLoaded {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for ModsLoaded {
    const ID: &'static str = "mods.loaded";
}

const _: () = assert!(core::mem::size_of::<ModsLoaded>() == 16);
