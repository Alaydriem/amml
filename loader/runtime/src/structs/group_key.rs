/// The name of one mesh group of the loader: `id` is never given twice in a process, `generation` counts the times the
/// book dropped every group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GroupKey {
    pub id: u32,
    pub generation: u32,
}
