use std::ffi::c_void;

use amml_runtime::CameraMath;

use crate::{CameraLayout, CameraLayoutError};

/// Reads the field of view and the aspect ratio of a `mce::Camera` from the top of its projection stack.
///
/// `Camera::getProjectionMatrix` is inline in the game, and the camera's own `mFov` and `mAspectRatio` stay 0 in the
/// render path, so the values come from the matrix (`CameraMath::projection`). The camera is read through
/// `CameraLayout`; `from_lookup` refuses a build whose `Camera.mForward` is not the mirror's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CameraProjection;

impl CameraProjection {
    /// The reader, when `Camera.mForward` in `lookup` is `CameraLayout::FORWARD`.
    pub fn from_lookup(lookup: &dyn Fn(&str) -> Option<u64>) -> Result<Self, CameraLayoutError> {
        let entry = "Camera.mForward";
        let found = lookup(entry).ok_or(CameraLayoutError::Missing { entry })?;
        if found != CameraLayout::FORWARD {
            return Err(CameraLayoutError::Mismatch { entry, expected: CameraLayout::FORWARD, found });
        }
        Ok(Self)
    }

    /// The vertical field of view in degrees and the aspect ratio; `None` for a null camera, an empty stack or a matrix
    /// without usable focal lengths.
    ///
    /// # Safety
    /// `camera` is null or a live `mce::Camera` of the build `from_lookup` checked, on the thread that owns it.
    pub unsafe fn read(&self, camera: *const c_void) -> Option<(f32, f32)> {
        if camera.is_null() {
            return None;
        }
        let layout = camera.cast::<CameraLayout>();
        // SAFETY: the caller guarantees a live Camera, laid out as `CameraLayout` (checked by the anchor).
        let top = unsafe { (*layout).projection_stack.stack.back() }?;
        // SAFETY: `top` is the last matrix of the live deque.
        let matrix = unsafe { top.read_unaligned() };
        CameraMath::projection(&matrix)
    }
}
