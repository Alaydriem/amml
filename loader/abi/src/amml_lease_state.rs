/// The state of an input lease.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlLeaseState(pub u8);

impl AmmlLeaseState {
    pub const ACTIVE: Self = Self(0);
    pub const SUSPENDED: Self = Self(1);
    pub const LOST: Self = Self(2);
}
