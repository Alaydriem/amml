use amml_abi::AmmlCameraKind;

/// The vanilla camera the client shows the world through (`CameraControl::active_camera`). A camera the game or a
/// server added, a free camera included, is `Other`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraKind {
    /// `minecraft:first_person`.
    FirstPerson,
    /// `minecraft:third_person`: from behind the player.
    ThirdPerson,
    /// `minecraft:third_person_front`: from in front of the player.
    ThirdPersonFront,
    /// Any other camera.
    Other,
}

impl CameraKind {
    pub(crate) fn from_abi(kind: AmmlCameraKind) -> Self {
        match kind {
            AmmlCameraKind::FIRST_PERSON => Self::FirstPerson,
            AmmlCameraKind::THIRD_PERSON => Self::ThirdPerson,
            AmmlCameraKind::THIRD_PERSON_FRONT => Self::ThirdPersonFront,
            _ => Self::Other,
        }
    }

    // The value of the kind in the ABI, for the fake loader.
    #[cfg(feature = "test")]
    pub(crate) fn to_abi(self) -> AmmlCameraKind {
        match self {
            Self::FirstPerson => AmmlCameraKind::FIRST_PERSON,
            Self::ThirdPerson => AmmlCameraKind::THIRD_PERSON,
            Self::ThirdPersonFront => AmmlCameraKind::THIRD_PERSON_FRONT,
            Self::Other => AmmlCameraKind::OTHER,
        }
    }
}
