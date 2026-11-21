/// Why the loader released an injected input.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlReleaseReason(pub u8);

impl AmmlReleaseReason {
    pub const FOCUS_LOST: Self = Self(0);
    pub const WORLD_LEFT: Self = Self(1);
    pub const SUSPENDED: Self = Self(2);
    pub const LEASE_ENDED: Self = Self(3);
    pub const REPLACED: Self = Self(4);
    pub const DISABLED: Self = Self(5);
    /// A real key or button up, or a key reset of the game, ended the injected hold in the game.
    pub const RESET: Self = Self(6);
}
