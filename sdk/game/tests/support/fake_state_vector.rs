/// The `StateVectorComponent` of a fake actor: the position sits after four padding bytes.
#[allow(dead_code)]
#[repr(C)]
pub struct FakeStateVector {
    pad: [u8; 4],
    pos: [f32; 3],
}

impl FakeStateVector {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { pad: [0; 4], pos: [x, y, z] }
    }
}
