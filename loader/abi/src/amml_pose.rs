use crate::{AmmlRotation, AmmlVec3};

/// A camera pose: the eye position in world coordinates, the view direction (the signs of `AmmlRotation`) and the
/// roll in degrees about the view direction (positive turns the picture clockwise as the viewer sees it).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct AmmlPose {
    pub position: AmmlVec3,
    pub rotation: AmmlRotation,
    pub roll: f32,
}

impl AmmlPose {
    /// Whether every number of the pose is finite.
    pub fn is_finite(&self) -> bool {
        let numbers =
            [self.position.x, self.position.y, self.position.z, self.rotation.yaw, self.rotation.pitch, self.roll];
        numbers.iter().all(|value| value.is_finite())
    }
}

const _: () = assert!(core::mem::size_of::<AmmlPose>() == 24);
