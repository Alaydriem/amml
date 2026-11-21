use crate::{AmmlEvent, AmmlEventHeader};

/// Runs once per rendered frame, before or after the game work as `header.phase` says.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FrameUpdate {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for FrameUpdate {
    const ID: &'static str = "frame.update";
}

const _: () = assert!(core::mem::size_of::<FrameUpdate>() == 16);
