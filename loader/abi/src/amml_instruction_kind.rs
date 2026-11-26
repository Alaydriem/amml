/// What a camera instruction asks of the client's camera. A plain integer so that a value from the other side of the
/// boundary can never be invalid. Zero is no kind.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlInstructionKind(pub u8);

impl AmmlInstructionKind {
    /// The instruction sets a camera preset (its set part is present and its clear part is not).
    pub const SET: Self = Self(1);
    /// The instruction clears the camera back to the player (its clear part is present and true, its set part is not).
    pub const CLEAR: Self = Self(2);
    /// Both parts, or neither: a fade, a field of view, a target or another option only.
    pub const OTHER: Self = Self(3);
}
