use crate::{AmmlEvent, AmmlEventHeader};

/// The game is about to end a frame (`MinecraftGame::endFrame`): raised Before the call, held clock or not, on the
/// game thread.
///
/// `count` is the loader's count of `endFrame` calls. The loader counts while a mod listens to this event or holds the
/// game clock; the count starts at 1 for the first counted call and never goes back.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FrameEnd {
    pub header: AmmlEventHeader,
    pub count: u64,
}

impl AmmlEvent for FrameEnd {
    const ID: &'static str = "time.frame_end";
}

const _: () = assert!(core::mem::size_of::<FrameEnd>() == 24);
