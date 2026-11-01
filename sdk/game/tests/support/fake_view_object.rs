/// A fake `ViewRenderObject`: 32 padding bytes, then the inline view data with the camera position after 8 bytes.
#[allow(dead_code)]
#[repr(C)]
pub struct FakeViewObject {
    pad: [u8; 32],
    view_pad: [f32; 2],
    camera_pos: [f32; 3],
}

impl FakeViewObject {
    pub fn new(camera_pos: [f32; 3]) -> Self {
        Self { pad: [0; 32], view_pad: [0.0; 2], camera_pos }
    }
}
