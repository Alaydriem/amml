/// What a mesh of the book is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshKind {
    /// A mesh of colour quads: the world material and the loader's white texture.
    Colour,
    /// A mesh of the block mesher: the client's blend-block material and the renderer's terrain atlas.
    Block,
}
