/// The name of one block entity ghost of the loader: `id` is never given twice in a process, `generation` counts the
/// times the book dropped every ghost.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GhostEntityKey {
    pub id: u32,
    pub generation: u32,
}
