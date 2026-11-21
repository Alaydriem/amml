/// How a held light fades with distance.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid; a value the
/// loader does not know is read as `ROUND`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AmmlLightFalloff(pub u8);

impl AmmlLightFalloff {
    /// By straight-line distance to the block centre, rounded to whole blocks.
    pub const ROUND: Self = Self(0);
    /// By block steps (taxicab distance), as a placed torch fades.
    pub const TAXICAB: Self = Self(1);
}
