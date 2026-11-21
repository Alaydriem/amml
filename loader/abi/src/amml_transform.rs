use crate::{AmmlBlockPos, AmmlVec3};

/// Where one draw of a world mesh goes: the mesh point (0, 0, 0) lands at `origin` + `offset` in world coordinates,
/// the mesh is turned by `yaw_degrees` about +Y (the signs of `AmmlRotation`: 90 turns +Z to -X) and scaled by
/// `scale` in each axis.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmlTransform {
    pub origin: AmmlBlockPos,
    pub offset: AmmlVec3,
    pub scale: f32,
    pub yaw_degrees: f32,
}

const _: () = assert!(core::mem::size_of::<AmmlTransform>() == 32);
