use std::ffi::c_void;
use std::fmt;

use crate::Client;
use crate::client_tables::ClientTables;

/// What `AmmlMod::disable` is told about the disable.
///
/// `game_alive` is false when the game has not started or is shutting down: the mod must not call
/// game actions then.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct DisableCtx {
    pub game_alive: bool,
    client_table: Option<ClientTables>,
    module: &'static str,
}

impl DisableCtx {
    /// A context with no client.
    pub fn new(game_alive: bool) -> Self {
        Self { game_alive, client_table: None, module: "" }
    }

    /// A context that can ask the loader for the current client.
    pub(crate) fn with_client(game_alive: bool, client_table: Option<ClientTables>, module: &'static str) -> Self {
        Self { game_alive, client_table, module }
    }

    /// The current primary client, while the game runs. `None` when the game does not run, there is no
    /// primary client, the disable runs off the game thread or the loader has no client table.
    pub fn client(&self) -> Option<Client<'_>> {
        if !self.game_alive {
            return None;
        }
        let table = self.client_table?;
        let mut out: *const c_void = std::ptr::null();
        // SAFETY: the loader's function only writes `out`, a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (table.client.current)(&mut out) };
        (status.is_ok() && !out.is_null()).then(|| Client::new(table, out, self.module))
    }
}

impl fmt::Debug for DisableCtx {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("DisableCtx").field("game_alive", &self.game_alive).finish()
    }
}
