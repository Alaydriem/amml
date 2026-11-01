/// The `ActorRotationComponent` of a fake actor: the rotation sits after eight padding bytes.
#[allow(dead_code)]
#[repr(C)]
pub struct FakeRotation {
    pad: [u8; 8],
    rot: [f32; 2],
}

impl FakeRotation {
    pub fn new(pitch: f32, yaw: f32) -> Self {
        Self { pad: [0; 8], rot: [pitch, yaw] }
    }
}
