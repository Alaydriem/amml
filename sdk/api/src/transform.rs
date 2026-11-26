use amml_abi::AmmlTransform;

use crate::{BlockPos, Vec3};

/// Where one draw of a world mesh goes: the mesh point (0, 0, 0) lands at `origin` + `offset` in world coordinates,
/// the mesh is turned by `yaw_degrees` about +Y with the signs of `Rotation` (90 turns +Z to -X) and scaled by `scale`
/// in each axis. The loader takes the camera position off in double precision, so a mesh far from the world origin
/// keeps its sub-block offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub origin: BlockPos,
    pub offset: Vec3,
    pub scale: f32,
    pub yaw_degrees: f32,
}

impl From<Transform> for AmmlTransform {
    fn from(transform: Transform) -> Self {
        Self {
            origin: transform.origin.into(),
            offset: transform.offset.into(),
            scale: transform.scale,
            yaw_degrees: transform.yaw_degrees,
        }
    }
}
