use amml_abi::ServerJoinStarting;

use crate::lent_strings::LentStrings;

/// Typed reads of a `ServerJoinStarting`: the game is about to start a join of a server, raised Before the game's call
/// once for every server join (the Servers tab, Realms, `join_server`, `join_server_direct`), not for a local world.
pub trait ServerJoinStartingExt {
    /// The resolved address of the connection info. Valid only inside the listener call: a copy of the event, or one
    /// kept past its call, gives `None`, as do an empty text and a text that is not UTF-8.
    fn host(&self) -> Option<&str>;

    /// The unresolved URL of the connection info (the text the user typed or the server list holds); the same rules
    /// as `host`.
    fn url(&self) -> Option<&str>;

    /// The server name: the external server name of the connection info, or the name the game passes when that is
    /// empty; the same rules as `host`.
    fn name(&self) -> Option<&str>;

    /// The port of the connection info; 0 when the loader has no layout of the info for this build.
    fn port(&self) -> u16;

    /// Whether the join is a server transfer.
    fn transfer(&self) -> bool;
}

impl ServerJoinStartingExt for ServerJoinStarting {
    fn host(&self) -> Option<&str> {
        LentStrings::find(self, 0)
    }

    fn url(&self) -> Option<&str> {
        LentStrings::find(self, 1)
    }

    fn name(&self) -> Option<&str> {
        LentStrings::find(self, 2)
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn transfer(&self) -> bool {
        self.transfer != 0
    }
}
