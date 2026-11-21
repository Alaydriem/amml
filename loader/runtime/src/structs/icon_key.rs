/// The name of one inline icon: `id` is never given twice in a process, `generation` counts the times the book freed
/// every icon, and `code_point` is the private-use character that draws it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IconKey {
    pub id: u64,
    pub generation: u32,
    pub code_point: u32,
}
