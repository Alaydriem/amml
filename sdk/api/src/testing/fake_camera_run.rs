use crate::{Pose, Projection, Vec3};

// The `CameraSetup` dispatch running now: its frame id, whether it is the After one, what the fake camera gives and the
// poses set in it.
pub(crate) struct FakeCameraRun {
    pub(crate) frame: u64,
    pub(crate) after: bool,
    pub(crate) projection: Option<Projection>,
    pub(crate) rendered: Option<Vec3>,
    pub(crate) poses: Vec<Pose>,
}
