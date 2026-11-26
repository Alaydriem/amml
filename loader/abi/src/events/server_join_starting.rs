use crate::{AmmlEvent, AmmlEventHeader, AmmlStr};

/// The game is about to start a join of a server: raised Before the game's call, on the game thread, once for every
/// server join. Version 2 (this layout) is raised from two game functions: `MinecraftGame::joinMultiplayerWithAddress`,
/// which the game's own server joins reach (the Servers tab, `join_server`, Realms) and which is not raised for a
/// local world (its local flag set), and `ClientInstance::startExternalNetworkWorld` (`join_server_direct`). A direct
/// join that the game passes on to `joinMultiplayerWithAddress` with the same address and port is raised once.
/// Version 1 was raised only through `startExternalNetworkWorld`.
///
/// `host` is the resolved address of the connection info, `url` its unresolved URL (the text the user typed or the
/// list holds) and `name` its external server name (the server list name); when that is empty, `name` is the server
/// name argument of the game's call. `port` is the port of the info (0 when the loader has no
/// layout of the info for this build). `transfer` is 1 when the join is a server transfer. An unreadable text is an
/// empty view. The three views are valid only inside the listener call.
///
/// The event context has no client (`client` is null and the thread is `GAME`): the loader raises it with its own
/// context, so a listener reads the joining client from the session (`client.read`), not from the context.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ServerJoinStarting {
    pub header: AmmlEventHeader,
    pub port: u16,
    pub transfer: u8,
    pub _reserved: [u8; 5],
    pub host: AmmlStr,
    pub url: AmmlStr,
    pub name: AmmlStr,
}

impl AmmlEvent for ServerJoinStarting {
    const ID: &'static str = "world.server_join_starting";
    const VERSION: u32 = 2;
}

const _: () = assert!(core::mem::size_of::<ServerJoinStarting>() == 72);
