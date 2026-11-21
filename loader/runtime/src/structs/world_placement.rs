/// Where one draw of a world mesh goes: the mesh point (0, 0, 0) lands at `origin` + `offset` in world coordinates,
/// turned by `yaw_degrees` about +Y and scaled by `scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldPlacement {
    pub origin: [i32; 3],
    pub offset: [f32; 3],
    pub scale: f32,
    pub yaw_degrees: f32,
}
