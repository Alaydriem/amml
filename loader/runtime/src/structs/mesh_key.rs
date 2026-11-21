/// The name of one mesh of the loader: `id` is never given twice in a process, `generation` counts the times the book
/// freed every mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshKey {
    pub id: u64,
    pub generation: u32,
}
