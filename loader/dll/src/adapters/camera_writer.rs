use std::ffi::c_void;
use std::mem::offset_of;

use amml_runtime::CameraBasis;

use crate::{CameraLayout, CameraLayoutError, ViewRenderDataLayout};

/// `void mce::Camera::updateViewMatrixDependencies()`.
type UpdateViewFn = unsafe extern "C" fn(*mut c_void);

/// Writes a camera pose into the game's render objects: the view matrix and basis of a `mce::Camera`, and the camera
/// position of a `ViewRenderObject` and of a `LevelRendererCamera`.
///
/// `from_lookup` checks every mirror against its entry first: the `mce::Camera` layout (`Camera.mForward`) and the
/// view data head (`LevelRendererCamera.mCameraTargetPos` 0Ch after `mCameraPos`, the run `createViewRenderObject`
/// copies into the view data). A mismatch names its entry; the loader then does not offer `camera.pose` on this build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CameraWriter {
    update: usize,
    view_pos: usize,
    level_pos: usize,
    level_target: usize,
}

impl CameraWriter {
    /// The writer from `lookup`, which gives the value of an entry (`None` when it does not resolve).
    pub fn from_lookup(lookup: &dyn Fn(&str) -> Option<u64>) -> Result<Self, CameraLayoutError> {
        let get = |entry: &'static str| lookup(entry).ok_or(CameraLayoutError::Missing { entry });
        let offset = |entry: &'static str| {
            let value = get(entry)?;
            usize::try_from(value).map_err(|_| CameraLayoutError::Mismatch { entry, expected: 0, found: value })
        };
        let forward = get("Camera.mForward")?;
        if forward != CameraLayout::FORWARD {
            return Err(CameraLayoutError::Mismatch {
                entry: "Camera.mForward",
                expected: CameraLayout::FORWARD,
                found: forward,
            });
        }
        let update = get("Camera.updateViewMatrixDependencies")?;
        let update =
            usize::try_from(update).ok().filter(|address| *address != 0).ok_or(CameraLayoutError::Mismatch {
                entry: "Camera.updateViewMatrixDependencies",
                expected: 1,
                found: update,
            })?;
        let level_pos = offset("LevelRendererCamera.mCameraPos")?;
        let level_target = offset("LevelRendererCamera.mCameraTargetPos")?;
        let target_step = offset_of!(ViewRenderDataLayout, camera_target_pos);
        if level_pos.checked_add(target_step) != Some(level_target) {
            return Err(CameraLayoutError::Mismatch {
                entry: "LevelRendererCamera.mCameraTargetPos",
                expected: level_pos as u64 + target_step as u64,
                found: level_target as u64,
            });
        }
        let view_data = offset("ViewRenderObject.mViewData")?;
        let camera_pos = offset("ViewRenderData.mCameraPos")?;
        let view_pos = view_data.checked_add(camera_pos).ok_or(CameraLayoutError::Mismatch {
            entry: "ViewRenderData.mCameraPos",
            expected: 0,
            found: camera_pos as u64,
        })?;
        Ok(Self { update, view_pos, level_pos, level_target })
    }

    /// Writes the view matrix of `basis` to the top of the camera's view stack and sets the stack's dirty flag, as
    /// `MatrixStack::getTop` does, then the inverse and the basis vectors, and calls
    /// `mce::Camera::updateViewMatrixDependencies`. Nothing is written for a null camera or an empty stack.
    ///
    /// # Safety
    /// `camera` is null or the live `mce::Camera` of a `setupCamera` call that just returned on this thread, of the
    /// build `from_lookup` checked.
    pub unsafe fn write_rotation(&self, camera: *mut c_void, basis: &CameraBasis) {
        if camera.is_null() {
            return;
        }
        let layout = camera.cast::<CameraLayout>();
        // SAFETY: the caller guarantees a live Camera, laid out as `CameraLayout` (checked by the anchor).
        let Some(top) = (unsafe { (*layout).view_stack.stack.back() }) else { return };
        // SAFETY: `top` is the last matrix of the live deque; the other places are members of the live Camera.
        unsafe {
            top.write_unaligned(basis.view);
            (&raw mut (*layout).view_stack.is_dirty).write(true);
            (&raw mut (*layout).inverse_view).write_unaligned(basis.inverse);
            (&raw mut (*layout).forward).write_unaligned(basis.forward);
            (&raw mut (*layout).right).write_unaligned(basis.right);
            (&raw mut (*layout).up).write_unaligned(basis.up);
        }
        // SAFETY: the entry is the address of `void mce::Camera::updateViewMatrixDependencies()`.
        let update: UpdateViewFn = unsafe { std::mem::transmute::<usize, UpdateViewFn>(self.update) };
        // SAFETY: `camera` is the live Camera the members were written to.
        unsafe { update(camera) };
    }

    /// Writes `position` to the camera position and target of the view data of `view_object`.
    ///
    /// # Safety
    /// `view_object` is null or the `ViewRenderObject` a `createViewRenderObject` call just filled, of the build
    /// `from_lookup` checked.
    pub unsafe fn write_view_position(&self, view_object: usize, position: [f32; 3]) {
        let target = self.view_pos + offset_of!(ViewRenderDataLayout, camera_target_pos);
        // SAFETY: forwarded to the caller.
        unsafe { Self::write_pair(view_object, self.view_pos, target, position) };
    }

    /// Writes `position` to `mCameraPos` and `mCameraTargetPos` of `level_camera`.
    ///
    /// # Safety
    /// `level_camera` is null or the live `LevelRendererCamera` of a `preRenderUpdate` call that just returned, of the
    /// build `from_lookup` checked.
    pub unsafe fn write_level_camera_position(&self, level_camera: usize, position: [f32; 3]) {
        // SAFETY: forwarded to the caller.
        unsafe { Self::write_pair(level_camera, self.level_pos, self.level_target, position) };
    }

    // Safety: a non-zero `object` is a live object with a Vec3 at each offset.
    unsafe fn write_pair(object: usize, first: usize, second: usize, position: [f32; 3]) {
        if object == 0 {
            return;
        }
        for at in [first, second] {
            let Some(address) = object.checked_add(at) else { return };
            // SAFETY: the caller guarantees a live object with a Vec3 at each offset.
            unsafe { std::ptr::with_exposed_provenance_mut::<[f32; 3]>(address).write_unaligned(position) };
        }
    }
}
