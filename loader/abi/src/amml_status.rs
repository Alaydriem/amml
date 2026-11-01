/// Result code of every call across the loader and mod boundary.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlStatus(pub i32);

impl AmmlStatus {
    pub const OK: Self = Self(0);
    pub const INVALID_ARGUMENT: Self = Self(1);
    pub const MISSING_ENTRY: Self = Self(2);
    pub const HOOK_FAILED: Self = Self(3);
    pub const VERSION_MISMATCH: Self = Self(4);
    pub const PANICKED: Self = Self(5);
    pub const FAILED: Self = Self(6);

    /// Whether the call succeeded. Any value other than `OK`, known or not, is a failure.
    pub fn is_ok(self) -> bool {
        self == Self::OK
    }
}
