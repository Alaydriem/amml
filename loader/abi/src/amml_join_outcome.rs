/// How the game answered a server join (`JoinServerResult::outcome`).
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. `OTHER` is not a
/// failure: the game gave an answer the loader does not name, and the join goes on.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlJoinOutcome(pub u32);

impl AmmlJoinOutcome {
    pub const OTHER: Self = Self(0);
    /// The game did not find the server.
    pub const NOT_FOUND: Self = Self(1);
    /// The host name did not resolve.
    pub const RESOLVE_FAILED: Self = Self(2);
    /// The game's connectivity test failed.
    pub const CONNECTIVITY_FAILED: Self = Self(3);
    /// The game refused the join.
    pub const REFUSED: Self = Self(4);
}
