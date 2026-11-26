/// What a `WindowPointer` event reports.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlPointerKind(pub u8);

impl AmmlPointerKind {
    /// The pointer moved inside the client area.
    pub const MOVE: Self = Self(0);
    /// A mouse button went down or up.
    pub const BUTTON: Self = Self(1);
    /// The wheel turned.
    pub const WHEEL: Self = Self(2);
    /// The pointer left the client area.
    pub const LEAVE: Self = Self(3);
}
