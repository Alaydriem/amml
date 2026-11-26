/// What a screenshot or capture holds.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlShotLayers(pub u8);

impl AmmlShotLayers {
    /// The game image, before any mod drew on it.
    pub const GAME_ONLY: Self = Self(0);
    /// The image as presented, with what mods drew.
    pub const WITH_OVERLAYS: Self = Self(1);
}
