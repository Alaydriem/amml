use amml_abi::AmmlInstructionKind;

/// What a camera instruction asks of the client's camera (`CameraInstructionReceivedExt::kind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstructionKind {
    /// It sets a camera preset.
    Set,
    /// It clears the camera back to the player.
    Clear,
    /// Both, or neither: a fade, a field of view, a target or another option only.
    Other,
}

impl InstructionKind {
    // A kind the loader gives that this crate does not know is `Other`.
    pub(crate) fn from_abi(kind: AmmlInstructionKind) -> Self {
        match kind {
            AmmlInstructionKind::SET => Self::Set,
            AmmlInstructionKind::CLEAR => Self::Clear,
            _ => Self::Other,
        }
    }

    // The value of the kind in the ABI, for the fake loader.
    #[cfg(feature = "test")]
    pub(crate) fn to_abi(self) -> AmmlInstructionKind {
        match self {
            Self::Set => AmmlInstructionKind::SET,
            Self::Clear => AmmlInstructionKind::CLEAR,
            Self::Other => AmmlInstructionKind::OTHER,
        }
    }
}
