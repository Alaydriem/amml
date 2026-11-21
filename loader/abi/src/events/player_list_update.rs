use crate::{AmmlEvent, AmmlEventHeader};

/// The server sent a player list packet and the game handled it (`network.player_list`, After). `action` is
/// `ADD` or `REMOVE` and `count` the number of entries of that action, at most 4096: the loader looks at the first
/// 4096 entries of the packet and keeps those of the kind of the first entry (entries of the other kind are left out
/// and one packet can mix both); the entries are read through
/// `AmmlNetworkTable::player_list_entry` and `player_list_text`, and only while the listeners run.
///
/// Raised on the thread that runs the game's handler (the context says which); the `network` reads refuse any call
/// made after the last listener returned.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PlayerListUpdate {
    pub header: AmmlEventHeader,
    pub action: u32,
    pub count: u32,
}

impl PlayerListUpdate {
    /// The entries add players.
    pub const ADD: u32 = 1;
    /// The entries remove players.
    pub const REMOVE: u32 = 2;
    /// Most entries one event gives; more are cut.
    pub const MAX_ENTRIES: u32 = 4096;
}

impl AmmlEvent for PlayerListUpdate {
    const ID: &'static str = "network.player_list";
}

const _: () = assert!(core::mem::size_of::<PlayerListUpdate>() == 24);
