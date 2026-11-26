use crate::{AmmlEvent, AmmlEventHeader, AmmlRouteVia};

/// The game asks for its idle screen (after about 15 minutes with no input), by a push or a replace of the idle
/// route (`via`). Raised only for the idle route, before the router takes it. A listener cancels it: the router is
/// not called and the game is told the route was not taken (`false`); the game may ask again later.
///
/// Raised on the thread of the router call: `GAME` when it is the game thread, else `RENDER`. `via` and `_reserved`
/// are the loader's: it writes them back after every listener.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct IdleScreenOpen {
    pub header: AmmlEventHeader,
    pub via: AmmlRouteVia,
    pub _reserved: u32,
}

impl AmmlEvent for IdleScreenOpen {
    const ID: &'static str = "client.idle_screen_open";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<IdleScreenOpen>() == 24);
