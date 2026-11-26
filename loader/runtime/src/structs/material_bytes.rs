/// The four bytes of a game `RenderMaterial` that the loader patches for a Vibrant Visuals world mesh: blend on or off,
/// the source and destination blend factors (`mce::BlendTarget`) and the depth write flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialBytes {
    pub enable_blend: u8,
    pub blend_source: u8,
    pub blend_destination: u8,
    pub depth_write: u8,
}
