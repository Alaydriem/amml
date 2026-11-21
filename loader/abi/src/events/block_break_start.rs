use crate::{AmmlBlockPos, AmmlEvent, AmmlEventHeader, AmmlFace};

/// A player hits a block to start mining it.
///
/// Raised on the game thread only, for every player; `is_local_player` is 1 for the local player of the
/// primary client. The context carries the primary client. A cancel stops the hit: the game gets "not handled" and no block is destroyed.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BlockBreakStart {
    pub header: AmmlEventHeader,
    pub pos: AmmlBlockPos,
    pub face: AmmlFace,
    pub is_local_player: u8,
    pub _reserved: [u8; 2],
}

impl AmmlEvent for BlockBreakStart {
    const ID: &'static str = "player.block_break_start";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<BlockBreakStart>() == 32);
