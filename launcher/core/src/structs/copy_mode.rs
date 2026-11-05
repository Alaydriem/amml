// How a world copy treats the target folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyMode {
    // the target is missing or empty: copy to a staging folder, then rename it
    Fresh,
    // the target holds files: copy only the files it lacks
    Merge,
}
