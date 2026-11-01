use std::path::{Path, PathBuf};

use amml_data::BuildId;

/// Names the cache file of one exe build and one recipe set.
pub struct CacheFile;

impl CacheFile {
    /// `<dir>/<build stem>-<recipes digest>.hcl`.
    pub fn path(dir: &Path, build: BuildId, digest: &str) -> PathBuf {
        dir.join(format!("{}-{digest}.hcl", build.file_stem()))
    }
}
