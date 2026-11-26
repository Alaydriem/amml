use amml_abi::AmmlSkullPose;

use crate::Face;

/// The pose of a skull ghost: the face of the block it stands on or hangs from (`Face::Up` for a skull on the floor, a
/// side face for a skull on a wall) and its turn on the floor in sixteenths of a full turn (0 to 15).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SkullPose {
    pub face: Face,
    pub turn: u8,
}

impl From<SkullPose> for AmmlSkullPose {
    fn from(pose: SkullPose) -> Self {
        Self { face: pose.face.into(), turn: pose.turn }
    }
}
