/// The name of one item draw copy of the loader: `id` is never given twice in a process, `generation` counts the
/// world leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemDrawKey {
    pub id: u64,
    pub generation: u32,
}
