use std::ffi::c_void;
use std::fmt;

use amml_abi::AmmlThread;

use crate::client_tables::ClientTables;
use crate::{Client, WorldEpoch};

/// What a listener is told beside the event: the game thread it runs on, whether the game runs, the
/// world epoch and the game client.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct EventCtx {
    pub thread: AmmlThread,
    pub game_alive: bool,
    pub epoch: WorldEpoch,
    client: *const c_void,
    client_table: Option<ClientTables>,
    module: &'static str,
}

impl EventCtx {
    /// A context with no client.
    pub fn new(thread: AmmlThread, game_alive: bool, epoch: WorldEpoch) -> Self {
        Self { thread, game_alive, epoch, client: std::ptr::null(), client_table: None, module: "" }
    }

    /// The same context with the event's client handle, the loader's client table and the mod id
    /// that client writes are made for.
    pub(crate) fn with_client(
        self,
        client: *const c_void,
        client_table: Option<ClientTables>,
        module: &'static str,
    ) -> Self {
        Self { client, client_table, module, ..self }
    }

    /// The game client of the event; `None` when the event has none (before the first client update,
    /// after a world leave) or the loader has no client table.
    pub fn client(&self) -> Option<Client<'_>> {
        let table = self.client_table?;
        (!self.client.is_null()).then(|| Client::new(table, self.client, self.module))
    }
}

impl fmt::Debug for EventCtx {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EventCtx")
            .field("thread", &self.thread)
            .field("game_alive", &self.game_alive)
            .field("epoch", &self.epoch)
            .field("client", &self.client)
            .finish()
    }
}
