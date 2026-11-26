/// The arguments of the running `LevelRenderer::renderLevel` call whose `WorldRendered` dispatch runs on this thread:
/// the level renderer, its screen context and the frame object. All zero outside a dispatch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderCall {
    pub renderer: usize,
    pub screen: usize,
    pub frame_object: usize,
}
