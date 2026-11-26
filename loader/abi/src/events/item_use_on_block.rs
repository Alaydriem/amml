use crate::{AmmlBlockPos, AmmlEvent, AmmlEventHeader, AmmlFace};

/// A player uses the held item on a block (a hoe on grass, flint and steel).
///
/// Raised on the game thread only, for every player; `is_local_player` is 1 for the local player of the
/// primary client. The context carries the primary client. A cancel stops the use: the game gets a result with no success and no swing.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ItemUseOnBlock {
    pub header: AmmlEventHeader,
    pub pos: AmmlBlockPos,
    pub face: AmmlFace,
    pub is_local_player: u8,
    pub _reserved: [u8; 2],
}

impl AmmlEvent for ItemUseOnBlock {
    const ID: &'static str = "player.item_use_on_block";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<ItemUseOnBlock>() == 32);
