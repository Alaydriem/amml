use std::marker::PhantomData;

use amml_abi::{AmmlCameraTable, AmmlPose, AmmlProjection, AmmlStr, AmmlVec3};

use crate::{ApiError, Pose, Projection, Vec3};

/// The camera of one `CameraSetup` dispatch: the calls that belong to the frame the game sets the camera up for. It is
/// built only from the event of a running listener call (`CameraSetupExt::camera`) and borrows it, so it cannot
/// outlive the call. The loader checks the frame id on every call and refuses a call outside the dispatch.
pub struct Camera<'f> {
    table: AmmlCameraTable,
    frame: u64,
    module: &'static str,
    _event: PhantomData<&'f ()>,
}

impl<'f> Camera<'f> {
    pub(crate) fn new(table: AmmlCameraTable, frame: u64, module: &'static str) -> Self {
        Self { table, frame, module, _event: PhantomData }
    }

    /// Shows the world from `pose` in this frame only: the loader writes the rotation after the game's camera setup,
    /// and the position into the view and the chunk sort later in the frame. A frame without a call shows the game
    /// camera, so call it in every frame you fly (in `CameraSetup` Before). `Err(Refused)` when another mod set the pose
    /// of this frame first, `Err(Loader(INVALID_ARGUMENT))` for a number that is not finite, `Err(NotAvailable)` from a
    /// loader without `Area::CAMERA_POSE`.
    #[allow(unsafe_code)]
    pub fn set_pose(&self, pose: Pose) -> Result<(), ApiError> {
        let pose = AmmlPose::from(pose);
        // SAFETY: the table function is the loader's; the module id and the pose live for the call.
        let status = unsafe { (self.table.set_pose)(AmmlStr::from_str(self.module), self.frame, &raw const pose) };
        ApiError::check(status)
    }

    /// The camera position the loader wrote in the camera frame before this one; `None` when it wrote none (no pose in
    /// that frame), or without `Area::CAMERA_POSE`.
    #[allow(unsafe_code)]
    pub fn rendered_position(&self) -> Option<Vec3> {
        let mut out = AmmlVec3::default();
        // SAFETY: the table function is the loader's; `out` is writable for the call.
        let status = unsafe { (self.table.rendered_position)(self.frame, &raw mut out) };
        status.is_ok().then(|| Vec3::from(out))
    }

    /// The vertical field of view and the aspect ratio of the camera the game set up, read from its projection matrix.
    /// Only in `CameraSetup` After: `None` in Before, without `Area::CAMERA_PROJECTION`, or for a matrix without usable
    /// focal lengths.
    #[allow(unsafe_code)]
    pub fn projection(&self) -> Option<Projection> {
        let mut out = AmmlProjection::default();
        // SAFETY: the table function is the loader's; `out` is writable for the call.
        let status = unsafe { (self.table.projection)(self.frame, &raw mut out) };
        status.is_ok().then(|| Projection::from(out))
    }
}
