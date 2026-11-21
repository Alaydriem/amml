use crate::{AmmlBlockPos, AmmlEvent, AmmlEventHeader, AmmlFace};

/// A player goes on mining a block; once per game tick while the mining lasts.
///
/// Raised on the game thread only, for every player; `is_local_player` is 1 for the local player of the
/// primary client. The context carries the primary client. A cancel stops this tick of the mining: the game gets "not handled" and no block is destroyed.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BlockBreakContinue {
    pub header: AmmlEventHeader,
    pub pos: AmmlBlockPos,
    pub face: AmmlFace,
    pub is_local_player: u8,
    pub _reserved: [u8; 2],
}

impl AmmlEvent for BlockBreakContinue {
    const ID: &'static str = "player.block_break_continue";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<BlockBreakContinue>() == 32);
