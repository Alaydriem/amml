use std::path::PathBuf;
use std::time::SystemTime;

// One file of the registry cache, counted together with its etag file.
#[derive(Debug, Clone)]
pub struct CachedFile {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
}
