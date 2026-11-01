use std::ffi::c_void;

use crate::game::CameraLayout;
use crate::{EntryId, GameEntries};

/// The field of view and aspect ratio of a `mce::Camera`, read from the top of its projection stack.
///
/// `Camera::getProjectionMatrix` is inline in the game (`*projectionMatrixStack->top()._m`), and the camera's own
/// `mFov` and `mAspectRatio` stay 0 in the render path, so the values come from the matrix: P[1][1] is the vertical
/// focal length and P[1][1] / P[0][0] the aspect ratio. The camera is read through `CameraLayout`, anchored by the
/// `Camera.mForward` entry.
pub struct Projection;

impl Projection {
    /// The feature of the read.
    pub const FEATURE: &'static str = "game.projection";

    /// The entries of `FEATURE`.
    pub const ENTRIES: [EntryId; 1] = [EntryId::CAMERA_FORWARD];

    /// The vertical field of view in degrees and the aspect ratio. `Some((0.0, 0.0))` for a matrix whose focal
    /// lengths are not finite and positive; `None` for a null camera, a missing entry, an anchor that does not match
    /// the layout or an empty stack. Render thread; the read is not thread-checked.
    ///
    /// # Safety
    /// `camera` must be null or a live `mce::Camera` of the game build the entries were resolved for.
    #[allow(unsafe_code)]
    pub unsafe fn read(entries: &GameEntries, camera: *const c_void) -> Option<(f32, f32)> {
        if entries.get(EntryId::CAMERA_FORWARD)? != CameraLayout::FORWARD || camera.is_null() {
            return None;
        }
        let layout = camera.cast::<CameraLayout>();
        // SAFETY: the caller guarantees a live Camera, laid out as `CameraLayout` (checked by the anchor above).
        let top = unsafe { (*layout).projection_stack.stack.back() }?;
        // SAFETY: `top` is the last matrix of the live deque.
        let matrix: [f32; 16] = unsafe { top.cast::<[f32; 16]>().read_unaligned() };
        let (focal_x, focal_y) = (matrix[0], matrix[5]);
        let usable = focal_x.is_finite() && focal_y.is_finite() && focal_x > 0.0 && focal_y > 0.0;
        if !usable {
            return Some((0.0, 0.0));
        }
        Some(((2.0 * (1.0 / focal_y).atan()).to_degrees(), focal_y / focal_x))
    }
}
