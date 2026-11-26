use amml_abi::CameraSetup;

use crate::Camera;
use crate::lent_camera::LentCamera;

/// Typed reads of a `CameraSetup`: the game sets up the camera of the primary client for a rendered frame, Before and
/// After its call (`event.header().phase`).
///
/// The event is raised only while a world is joined, on the thread that runs the game's camera setup: the game thread
/// or the render thread (`EventCtx::thread`).
pub trait CameraSetupExt {
    /// The id of the game frame. It changes once per frame; Before and After of one setup share it.
    fn frame_id(&self) -> u64;

    /// Whether this is the first camera setup of `frame_id`. A game that sets the camera up more than once in a frame
    /// raises the later setups with `false`.
    fn first_in_frame(&self) -> bool;

    /// The camera of this frame: `None` when the loader has no camera table, and for any `CameraSetup` that is not the
    /// event of a running listener call on this thread (a copy, or one kept past its call).
    fn camera(&self) -> Option<Camera<'_>>;
}

impl CameraSetupExt for CameraSetup {
    fn frame_id(&self) -> u64 {
        self.frame_id
    }

    fn first_in_frame(&self) -> bool {
        self.first_in_frame != 0
    }

    fn camera(&self) -> Option<Camera<'_>> {
        let lent = LentCamera::find(std::ptr::from_ref(self))?;
        Some(Camera::new(lent.table, self.frame_id, lent.module))
    }
}
