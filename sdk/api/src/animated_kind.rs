/// What an animated image of a skin is for (the game's `AnimatedTextureType`).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimatedKind {
    /// The animated face.
    Face,
    /// The animated body, 32 by 32 pixels a frame.
    Body32x32,
    /// The animated body, 128 by 128 pixels a frame.
    Body128x128,
    /// A kind this crate does not name; the value is the game's.
    Other(u32),
}

impl From<u32> for AnimatedKind {
    fn from(kind: u32) -> Self {
        match kind {
            1 => Self::Face,
            2 => Self::Body32x32,
            3 => Self::Body128x128,
            other => Self::Other(other),
        }
    }
}

impl From<AnimatedKind> for u32 {
    fn from(kind: AnimatedKind) -> Self {
        match kind {
            AnimatedKind::Face => 1,
            AnimatedKind::Body32x32 => 2,
            AnimatedKind::Body128x128 => 3,
            AnimatedKind::Other(other) => other,
        }
    }
}
