use crate::{LoaderState, ObjectRead, RenderCall};

/// Reads the camera position of the world view of a frame: the view in `FrameRenderObject.mViews` that belongs to the
/// running level renderer.
///
/// `mViews` is a vector of inline `ViewRenderObject`s (begin at `mViews`, end at `mViews + 8`, `ViewRenderObject.size`
/// bytes each); a view's `mViewData` is an inline `ViewRenderData` whose `mCameraPos` is three floats. The renderer's
/// own camera position is `LevelRendererCamera.mCameraPos` of the `LevelRendererPlayer` that
/// `LevelRenderer.mLevelRendererPlayer` points to (its `LevelRendererCamera` base is at 0). The reader gives the
/// position of the first view whose `mCameraPos` equals the renderer's, and nothing when no view does.
pub struct ViewCameraReader {
    views_at: usize,
    view_data_at: usize,
    camera_pos_at: usize,
    view_size: usize,
    player_at: usize,
    renderer_camera_at: usize,
}

impl ViewCameraReader {
    // The end pointer of the views vector sits right after its begin pointer.
    const VECTOR_END_AT: usize = 8;
    // The most views one frame is walked for.
    const MAX_VIEWS: usize = 16;
    // Three floats.
    const POSITION_BYTES: usize = 12;

    /// The reader of the offsets the build derived; `None` when one entry does not resolve or the view size cannot
    /// hold the position.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let offset = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let reader = Self {
            views_at: offset("FrameRenderObject.mViews")?,
            view_data_at: offset("ViewRenderObject.mViewData")?,
            camera_pos_at: offset("ViewRenderData.mCameraPos")?,
            view_size: offset("ViewRenderObject.size")?,
            player_at: offset("LevelRenderer.mLevelRendererPlayer")?,
            renderer_camera_at: offset("LevelRendererCamera.mCameraPos")?,
        };
        let position_end = reader.view_data_at.checked_add(reader.camera_pos_at)?.checked_add(Self::POSITION_BYTES)?;
        (position_end <= reader.view_size).then_some(reader)
    }

    /// The camera position of the view of `call.frame_object` that belongs to `call.renderer`. `None` for a null
    /// object or renderer, no matching view, or a position that is not finite.
    ///
    /// # Safety
    /// `call` holds zero or the renderer and frame object the game passed to the running `renderLevel` call, on the
    /// thread of that call.
    pub unsafe fn read(&self, call: RenderCall) -> Option<[f32; 3]> {
        // SAFETY: the caller guarantees a live renderer; its player pointer is at the derived offset.
        let player = unsafe { ObjectRead::pointer(call.renderer, self.player_at) }?;
        // SAFETY: the player is a live `LevelRendererPlayer`, whose camera base holds `mCameraPos` at the offset.
        let wanted = unsafe { ObjectRead::value::<[f32; 3]>(player, self.renderer_camera_at) }?;
        if !wanted.iter().all(|part| part.is_finite()) {
            return None;
        }
        let vector = call.frame_object.checked_add(self.views_at)?;
        // SAFETY: the caller guarantees the frame object is live; `mViews` holds the begin and end pointers.
        let (begin, end) = unsafe {
            (
                ObjectRead::value::<usize>(call.frame_object, self.views_at)?,
                ObjectRead::value::<usize>(vector, Self::VECTOR_END_AT)?,
            )
        };
        if begin == 0 || end <= begin {
            return None;
        }
        let count = ((end - begin) / self.view_size).min(Self::MAX_VIEWS);
        (0..count).find_map(|index| {
            let view = begin.checked_add(index.checked_mul(self.view_size)?)?;
            // SAFETY: the view lies inside the vector; its `mViewData` starts with `mCameraPos` (checked against the
            // view size in `new`).
            let position = unsafe { ObjectRead::value::<[f32; 3]>(view, self.view_data_at + self.camera_pos_at) }?;
            (position == wanted).then_some(position)
        })
    }
}
