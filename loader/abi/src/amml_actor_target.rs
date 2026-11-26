/// The actor a call about an actor's body names (`CameraControl::hide_actor`).
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. Zero is not a target.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlActorTarget(pub u8);

impl AmmlActorTarget {
    /// The local player of the primary client.
    pub const LOCAL_PLAYER: Self = Self(1);
}
