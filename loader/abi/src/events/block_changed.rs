use crate::{AmmlBlockPos, AmmlEvent, AmmlEventHeader};

/// The game changed a block in the dimension of the local player, after the change.
///
/// `dimension` is the id of the block source that changed; `layer` is the block layer (0 the main layer, 1 the liquid
/// layer); `new` and `old` are block type ids of the world calls (0 when the game gave no readable type). A change
/// whose `new` and `old` are the same type (a state change of one type) is raised too. Raised on the game thread
/// at once when the game changed the block there, else at the next `FrameUpdate` After.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BlockChanged {
    pub header: AmmlEventHeader,
    pub dimension: i32,
    pub pos: AmmlBlockPos,
    pub layer: u32,
    pub new: u32,
    pub old: u32,
}

impl AmmlEvent for BlockChanged {
    const ID: &'static str = "world.block_changed";
}

const _: () = assert!(core::mem::size_of::<BlockChanged>() == 44);
