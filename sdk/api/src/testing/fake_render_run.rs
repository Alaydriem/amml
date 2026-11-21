use crate::{MeshId, Vec3};

// The `WorldRendered` dispatch running now: its frame id, the camera position it gives, the meshes freed in it and the
// `build` and `draw` lines it accepted.
pub(crate) struct FakeRenderRun {
    pub(crate) frame: u64,
    pub(crate) camera_position: Option<Vec3>,
    pub(crate) freed: Vec<MeshId>,
    pub(crate) calls: Vec<String>,
}
