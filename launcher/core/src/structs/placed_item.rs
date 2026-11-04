use std::path::PathBuf;

// A file or folder moved into an instance, with the one it replaced (renamed aside) so the move can be undone.
#[derive(Debug, Clone)]
pub struct PlacedItem {
    pub target: PathBuf,
    pub replaced: Option<PathBuf>,
}
