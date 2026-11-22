use crate::{AmmlEvent, AmmlEventHeader, AmmlStr};

/// Runs when the client starts to join a world.
///
/// Version 2 adds the facts of the join. `local` is 1 for a join of a local world and 0 for any other. `server_name` is
/// the second `std::string const&` argument of the game's `onStartJoinGame` (r9) and `world_name` the third (at
/// entry+28h); the first (r8) is a correlation id that is not given. The roles come from the recipe note of the game
/// function, the earlier dejavu detour and the game image on both builds; the loader logs the three strings once per
/// join (`join started names`) and the in-game check L-D6 confirms them. An empty text is an empty view. Both views are
/// valid only inside the listener call.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JoinStarted {
    pub header: AmmlEventHeader,
    pub local: u8,
    pub _reserved: [u8; 7],
    pub server_name: AmmlStr,
    pub world_name: AmmlStr,
}

impl AmmlEvent for JoinStarted {
    const ID: &'static str = "world.join_started";
    const VERSION: u32 = 2;
}

const _: () = assert!(core::mem::size_of::<JoinStarted>() == 56);
