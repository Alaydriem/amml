/// How the client is connected to the world it plays. A plain integer so that a value from the other side of the
/// boundary can never be invalid. Zero is no kind.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlConnectionKind(pub u8);

impl AmmlConnectionKind {
    /// The world runs on this machine.
    pub const LOCAL: Self = Self(1);
    /// The world runs on another machine, named by a host.
    pub const REMOTE: Self = Self(2);
    /// Any other connection, or one the loader cannot tell.
    pub const OTHER: Self = Self(3);
}
