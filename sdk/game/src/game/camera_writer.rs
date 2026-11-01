use std::ffi::c_void;

use crate::game::{CameraLayout, ViewRenderDataLayout};
use crate::{EntryId, GameEntries};

/// Writes a camera pose into the game's render objects: the view matrix and basis of a `mce::Camera`, and the
/// camera positions of a `ViewRenderObject` and a `LevelRendererCamera`.
///
/// The camera is written through `CameraLayout`, anchored by the `Camera.mForward` entry; the other offsets and the
/// functions come from the `game.camera_write` entries. `new` refuses unless every one of them resolved and the
/// anchor matches the layout, so a write is never partial. The methods take no latch: check your `StopLatch` before
/// you call, because a render hook can still run after `disable`. Render thread.
pub struct CameraWriter {
    update: usize,
    view_pos: usize,
    level_pos: usize,
    level_target: usize,
}

impl CameraWriter {
    /// The feature that holds every write entry.
    pub const FEATURE: &'static str = "game.camera_write";

    /// The entries of `FEATURE`, the hook of `LevelRendererCamera::preRenderUpdate` among them.
    pub const ENTRIES: [EntryId; 7] = [
        EntryId::CAMERA_FORWARD,
        EntryId::CAMERA_UPDATE_VIEW_MATRIX_DEPENDENCIES,
        EntryId::LEVEL_RENDERER_CAMERA_PRE_RENDER_UPDATE,
        EntryId::LEVEL_RENDERER_CAMERA_CAMERA_POS,
        EntryId::LEVEL_RENDERER_CAMERA_CAMERA_TARGET_POS,
        EntryId::VIEW_RENDER_OBJECT_VIEW_DATA,
        EntryId::VIEW_RENDER_DATA_CAMERA_POS,
    ];

    /// The writer; `None` when any entry of `FEATURE` is missing or does not fit `usize`, or when `Camera.mForward`
    /// is not where `CameraLayout` has it.
    pub fn new(entries: &GameEntries) -> Option<Self> {
        let get = |id: EntryId| usize::try_from(entries.get(id)?).ok();
        if Self::ENTRIES.iter().any(|id| get(*id).is_none())
            || entries.get(EntryId::CAMERA_FORWARD)? != CameraLayout::FORWARD
        {
            return None;
        }
        let view_data = get(EntryId::VIEW_RENDER_OBJECT_VIEW_DATA)?;
        Some(Self {
            update: get(EntryId::CAMERA_UPDATE_VIEW_MATRIX_DEPENDENCIES)?,
            view_pos: view_data.checked_add(get(EntryId::VIEW_RENDER_DATA_CAMERA_POS)?)?,
            level_pos: get(EntryId::LEVEL_RENDERER_CAMERA_CAMERA_POS)?,
            level_target: get(EntryId::LEVEL_RENDERER_CAMERA_CAMERA_TARGET_POS)?,
        })
    }

    /// Writes `view` (column major) to the top of the camera's view stack and sets the stack's dirty flag, as
    /// `MatrixStack::getTop` does, then writes `inverse` and the basis vectors and calls
    /// `mce::Camera::updateViewMatrixDependencies`. Nothing is written for a null camera or an empty stack.
    ///
    /// # Safety
    /// `camera` must be null or a live `mce::Camera` of the game build the entries were resolved for, on the
    /// render thread that owns it.
    #[allow(unsafe_code)]
    pub unsafe fn write_rotation(
        &self,
        camera: *mut c_void,
        forward: [f32; 3],
        right: [f32; 3],
        up: [f32; 3],
        view: [[f32; 4]; 4],
        inverse: [[f32; 4]; 4],
    ) {
        if camera.is_null() || self.update == 0 {
            return;
        }
        let layout = camera.cast::<CameraLayout>();
        // SAFETY: the caller guarantees a live Camera, laid out as `CameraLayout` (checked by the anchor in `new`).
        let Some(top) = (unsafe { (*layout).view_stack.stack.back() }) else { return };
        // SAFETY: `top` is the last matrix of the live deque; the other places are members of the live Camera.
        unsafe {
            top.write_unaligned(view);
            (&raw mut (*layout).view_stack.is_dirty).write(true);
            (&raw mut (*layout).inverse_view).write_unaligned(inverse);
            (&raw mut (*layout).forward).write_unaligned(forward);
            (&raw mut (*layout).right).write_unaligned(right);
            (&raw mut (*layout).up).write_unaligned(up);
        }
        // SAFETY: the entry is the address of `void mce::Camera::updateViewMatrixDependencies()`.
        let update: unsafe extern "C" fn(*mut c_void) = unsafe { std::mem::transmute(self.update) };
        // SAFETY: `camera` is the live Camera the members were written to.
        unsafe { update(camera) };
    }

    /// Writes `position` to `mCameraPos` and `mCameraTargetPos` of the inline `ViewRenderData` of `view_object`.
    ///
    /// # Safety
    /// `view_object` must be null or a live `ViewRenderObject` of the game build the entries were resolved for.
    #[allow(unsafe_code)]
    pub unsafe fn write_view_position(&self, view_object: *mut c_void, position: [f32; 3]) {
        // SAFETY: forwarded to the caller.
        unsafe {
            Self::write_pair(
                view_object,
                self.view_pos,
                self.view_pos + std::mem::offset_of!(ViewRenderDataLayout, camera_target_pos),
                position,
            )
        };
    }

    /// Writes `position` to `mCameraPos` and `mCameraTargetPos` of `level_camera`.
    ///
    /// # Safety
    /// `level_camera` must be null or a live `LevelRendererCamera` of the game build the entries were resolved for.
    #[allow(unsafe_code)]
    pub unsafe fn write_level_camera_position(&self, level_camera: *mut c_void, position: [f32; 3]) {
        // SAFETY: forwarded to the caller.
        unsafe { Self::write_pair(level_camera, self.level_pos, self.level_target, position) };
    }

    #[allow(unsafe_code)]
    unsafe fn write_pair(object: *mut c_void, first: usize, second: usize, position: [f32; 3]) {
        if object.is_null() {
            return;
        }
        let base = object.cast::<u8>();
        // SAFETY: the caller guarantees a live object with a Vec3 at each offset.
        unsafe {
            base.add(first).cast::<[f32; 3]>().write_unaligned(position);
            base.add(second).cast::<[f32; 3]>().write_unaligned(position);
        }
    }
}
