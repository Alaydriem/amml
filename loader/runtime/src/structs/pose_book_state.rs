use amml_abi::AmmlVec3;

use crate::structs::held_pose::HeldPose;

/// What the pose book holds: the current camera frame, its pose, the position written in it and the position written
/// in the camera frame before it.
pub(crate) struct PoseBookState {
    pub(crate) frame: u64,
    pub(crate) held: Option<HeldPose>,
    pub(crate) written: Option<AmmlVec3>,
    pub(crate) rendered: Option<AmmlVec3>,
}
