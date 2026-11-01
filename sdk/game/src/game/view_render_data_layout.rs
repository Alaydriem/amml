use std::mem::offset_of;

/// Layout mirror of the head of the game's `ViewRenderData`: the camera position, then the camera target position.
/// It sits at `ViewRenderObject.mViewData` + `ViewRenderData.mCameraPos`.
#[repr(C)]
pub struct ViewRenderDataLayout {
    pub camera_pos: [f32; 3],        // 0x00 mCameraPos
    pub camera_target_pos: [f32; 3], // 0x0C mCameraTargetPos
}

const _: () = assert!(offset_of!(ViewRenderDataLayout, camera_target_pos) == 0x0C);
