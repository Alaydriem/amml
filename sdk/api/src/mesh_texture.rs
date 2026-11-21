/// The texture a world mesh is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeshTexture {
    /// The loader's white texture: the quad colours only.
    White,
    /// The game's terrain atlas. Not served in this version (`NotAvailable`).
    TerrainAtlas,
}
