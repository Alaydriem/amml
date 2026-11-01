use std::ffi::c_void;

use crate::game::RawObject;
use crate::{EntryId, GameEntries};

/// Reads of the camera values a render hook sees.
///
/// The reads take no latch. Check your `StopLatch` before you call: a render hook can still run after `disable`.
pub struct RenderCamera<'a> {
    entries: &'a GameEntries,
}

impl<'a> RenderCamera<'a> {
    /// A reader over the resolved `entries`.
    pub fn new(entries: &'a GameEntries) -> Self {
        Self { entries }
    }

    /// The forward vector of `camera`. `None` for a null pointer or when the entry is off. Render thread; the read
    /// is not thread-checked.
    ///
    /// Check your `StopLatch` before you call.
    ///
    /// # Safety
    /// `camera` must be null or a live `Camera` of the game build the entries were resolved for.
    #[allow(unsafe_code)]
    pub unsafe fn forward(&self, camera: *const c_void) -> Option<[f32; 3]> {
        let offset = self.entries.get(EntryId::CAMERA_FORWARD)?;
        if camera.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees a live Camera, and the entry is the offset of its `mForward` Vec3.
        unsafe { RawObject::read::<[f32; 3]>(camera, offset) }
    }

    /// The camera position of the `ViewRenderObject` `view_object`, from its inline `ViewRenderData`. `None` for a
    /// null pointer or when an entry is off. Render thread; the read is not thread-checked.
    ///
    /// Check your `StopLatch` before you call.
    ///
    /// # Safety
    /// `view_object` must be null or a live `ViewRenderObject` of the game build the entries were resolved for.
    #[allow(unsafe_code)]
    pub unsafe fn view_position(&self, view_object: *const c_void) -> Option<[f32; 3]> {
        let view_data = self.entries.get(EntryId::VIEW_RENDER_OBJECT_VIEW_DATA)?;
        let camera_pos = self.entries.get(EntryId::VIEW_RENDER_DATA_CAMERA_POS)?;
        if view_object.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees a live ViewRenderObject; `mViewData` is inline, so `mCameraPos` is at the
        // sum of the two offsets.
        unsafe { RawObject::read::<[f32; 3]>(view_object, view_data.checked_add(camera_pos)?) }
    }
}
