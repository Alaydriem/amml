/// The name of one texture of the loader: `id` is never given twice in a process, `generation` counts the times
/// the book freed every texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureKey {
    pub id: u64,
    pub generation: u32,
}
