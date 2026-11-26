use std::mem::offset_of;

/// Layout mirror of the head of the game's `ViewRenderData`: the camera position, then the camera target position. It
/// sits at `ViewRenderObject.mViewData` + `ViewRenderData.mCameraPos`.
///
/// Survey 8.3: `createViewRenderObject` copies `LevelRendererCamera.mCameraPos` and the target that follows it as one
/// run of 24 bytes into the start of the view object on 26.63 and 70.26, so the target follows the position here as it
/// does in the level camera. The camera writer checks that the two level camera entries are 0Ch apart.
#[repr(C)]
pub struct ViewRenderDataLayout {
    pub camera_pos: [f32; 3],        // 0x00 mCameraPos
    pub camera_target_pos: [f32; 3], // 0x0C mCameraTargetPos
}

const _: () = assert!(offset_of!(ViewRenderDataLayout, camera_target_pos) == 0x0C);
