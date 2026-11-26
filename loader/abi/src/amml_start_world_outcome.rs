/// How the game answered a local world start (`ClientStartLocalWorldFn`).
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. The game's "not ready"
/// answer is the status `NOT_READY`, not a value here.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlStartWorldOutcome(pub u32);

impl AmmlStartWorldOutcome {
    /// The game took the start.
    pub const STARTED: Self = Self(0);
    /// The world is not in the game's world list.
    pub const NOT_IN_LEVEL_LIST: Self = Self(1);
    /// Another world start has not finished.
    pub const STARTUP_IN_PROGRESS: Self = Self(2);
    /// A newer game version saved the world.
    pub const SAVED_IN_NEWER_VERSION: Self = Self(3);
    /// The world has platform locked content that the player did not confirm.
    pub const PLATFORM_LOCKED_CONTENT: Self = Self(4);
    /// The world folder is not on disk.
    pub const NOT_FOUND_IN_FILE_SYSTEM: Self = Self(5);
    /// The world has slim cache data.
    pub const SLIM_CACHE_DATA: Self = Self(6);
    /// Any other refusal of the game.
    pub const OTHER_REFUSAL: Self = Self(7);
}
