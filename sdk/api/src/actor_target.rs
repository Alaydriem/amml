use amml_abi::AmmlActorTarget;

/// The actor a call about an actor's body names (`CameraControl::hide_actor`).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorTarget {
    /// The local player of the primary client.
    LocalPlayer,
}

impl ActorTarget {
    pub(crate) fn to_abi(self) -> AmmlActorTarget {
        match self {
            Self::LocalPlayer => AmmlActorTarget::LOCAL_PLAYER,
        }
    }
}
