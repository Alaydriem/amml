use crate::{AmmlEvent, AmmlEventHeader};

/// A player uses the held item in the air (draws a bow, casts a rod).
///
/// Raised on the game thread only, for every player; `is_local_player` is 1 for the local player of the
/// primary client. The context carries the primary client. A cancel stops the use: the game gets "not handled".
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ItemUse {
    pub header: AmmlEventHeader,
    pub is_local_player: u8,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for ItemUse {
    const ID: &'static str = "player.item_use";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<ItemUse>() == 24);
