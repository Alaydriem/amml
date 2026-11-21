use crate::{AmmlEvent, AmmlEventHeader};

/// The server sent the new skin of a player and the game handled it (`network.player_skin`, After). The player and
/// the skin are read through `AmmlNetworkTable::player_skin`, and only while the listeners run.
///
/// Raised on the thread that runs the game's handler (the context says which).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PlayerSkinChanged {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for PlayerSkinChanged {
    const ID: &'static str = "network.player_skin";
}

const _: () = assert!(core::mem::size_of::<PlayerSkinChanged>() == core::mem::size_of::<AmmlEventHeader>());
