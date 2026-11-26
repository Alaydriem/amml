/// The images queued in the render context by the loader and not drawn yet: one texture drawn with one tint. The
/// game draws a whole queue with the texture of its last item, so one flush covers one texture only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawBatch {
    /// The address of the `BedrockTextureData` the queued items point to.
    pub texture: usize,
    pub tint: [f32; 3],
    pub alpha: f32,
}
