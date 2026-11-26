use crate::{AmmlEvent, AmmlEventHeader};

/// Runs once per game tick of the primary client; `tick` counts game ticks.
///
/// Version 2: the loader raises it only for the ticks of the primary client (the client of
/// `ClientInstance::onTick`), and the event context carries that client, so `EventCtx::client` gives it.
/// Version 1 was raised for the ticks of every client, with no client in the context. The layout is the same.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GameTick {
    pub header: AmmlEventHeader,
    pub tick: u64,
}

impl AmmlEvent for GameTick {
    const ID: &'static str = "client.game_tick";
    const VERSION: u32 = 2;
}

const _: () = assert!(core::mem::size_of::<GameTick>() == 24);
