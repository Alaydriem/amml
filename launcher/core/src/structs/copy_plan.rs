use crate::CopyMode;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

// The files one world copy moves, measured before the copy starts.
#[derive(Debug, Clone)]
pub struct CopyPlan {
    pub source: PathBuf,
    // `<data_dir>\Minecraft Bedrock`
    pub target: PathBuf,
    pub mode: CopyMode,
    // paths relative to `source`, with sizes; in `Merge` mode only the files the target lacks
    pub files: Vec<(PathBuf, u64)>,
    pub total_bytes: u64,
    // set by a `Fresh` copy once it made the target; `undo` removes nothing before that
    pub created: Arc<AtomicBool>,
}
