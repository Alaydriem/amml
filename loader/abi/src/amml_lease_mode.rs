/// How a lease shares input with other leases.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlLeaseMode(pub u8);

impl AmmlLeaseMode {
    pub const SHARED: Self = Self(0);
    pub const EXCLUSIVE: Self = Self(1);
}
