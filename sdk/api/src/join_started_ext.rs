use amml_abi::JoinStarted;

use crate::lent_strings::LentStrings;

/// Typed reads of a `JoinStarted` (version 2): the client starts to join a world. A loader of version 1 does not offer
/// version 2, so a listen for `JoinStarted` fails there.
pub trait JoinStartedExt {
    /// Whether the join is a join of a local world.
    fn local(&self) -> bool;

    /// The server name: the second `std::string const&` argument of `onStartJoinGame` (r9). Valid only inside the
    /// listener call: a copy of the event, or one kept past its call, gives `None`, as do an empty text and a text that
    /// is not UTF-8. The roles come from the recipe note of the game function, the earlier dejavu detour and the game
    /// image; the loader logs the three strings of the call once per join (`join started names`) and the in-game check
    /// L-D6 confirms them.
    fn server_name(&self) -> Option<&str>;

    /// The world name: the third `std::string const&` argument of `onStartJoinGame` (at entry+28h); the same rules as
    /// `server_name`.
    fn world_name(&self) -> Option<&str>;
}

impl JoinStartedExt for JoinStarted {
    fn local(&self) -> bool {
        self.local != 0
    }

    fn server_name(&self) -> Option<&str> {
        LentStrings::find(self, 0)
    }

    fn world_name(&self) -> Option<&str> {
        LentStrings::find(self, 1)
    }
}
