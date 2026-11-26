/// The texture a world mesh is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeshTexture {
    /// The loader's white texture: the quad colours only.
    White,
    /// The game's terrain atlas, drawn with the client's blend-block material (`MeshStyle::GHOST`). Colour quads carry
    /// no UV, so a build from them gives `NotAvailable`; the block mesher makes these meshes.
    TerrainAtlas,
}
