/// Whether an event is delivered before or after the game's own work.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlPhase(pub u8);

impl AmmlPhase {
    pub const BEFORE: Self = Self(0);
    pub const AFTER: Self = Self(1);
}
