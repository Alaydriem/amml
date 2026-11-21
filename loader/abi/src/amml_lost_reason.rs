/// Why a lease was lost.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlLostReason(pub u8);

impl AmmlLostReason {
    pub const PREEMPTED: Self = Self(0);
}
