use amml_abi::AmmlPose;

use crate::{Rotation, Vec3};

/// A camera pose: the eye position in world coordinates, the view direction (the signs of `Rotation`) and the roll in
/// degrees about the view direction. A positive roll turns the picture clockwise as the player sees it: at 90 the top
/// of the picture points where its right side pointed.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Pose {
    pub position: Vec3,
    pub rotation: Rotation,
    pub roll: f32,
}

impl From<Pose> for AmmlPose {
    fn from(pose: Pose) -> Self {
        Self { position: pose.position.into(), rotation: pose.rotation.into(), roll: pose.roll }
    }
}

impl From<AmmlPose> for Pose {
    fn from(pose: AmmlPose) -> Self {
        Self { position: pose.position.into(), rotation: pose.rotation.into(), roll: pose.roll }
    }
}
