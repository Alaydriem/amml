use std::path::{Path, PathBuf};

use amml_data::BuildId;

/// Names the cache file of one exe build, and removes the files of the older naming.
pub struct CacheFile;

impl CacheFile {
    /// `<dir>/<build stem>.hcl`.
    pub fn path(dir: &Path, build: BuildId) -> PathBuf {
        dir.join(format!("{}.hcl", build.file_stem()))
    }

    /// Deletes every `<8 hex>-<16 hex>-<16 hex>.hcl` file in `dir` (a cache file named by build and recipe digest)
    /// and returns how many it deleted. A missing or unreadable folder deletes nothing.
    pub fn remove_stale(dir: &Path) -> usize {
        let Ok(items) = std::fs::read_dir(dir) else {
            return 0;
        };
        let mut removed = 0;
        for item in items.flatten() {
            let is_file = item.file_type().is_ok_and(|kind| kind.is_file());
            let name = item.file_name();
            if is_file && name.to_str().is_some_and(Self::is_stale_name) && std::fs::remove_file(item.path()).is_ok() {
                removed += 1;
            }
        }
        removed
    }

    fn is_stale_name(name: &str) -> bool {
        let Some(stem) = name.strip_suffix(".hcl") else {
            return false;
        };
        let parts: Vec<&str> = stem.split('-').collect();
        let hex = |part: &str, len: usize| part.len() == len && part.bytes().all(|byte| byte.is_ascii_hexdigit());
        matches!(parts.as_slice(), [timestamp, text_hash, digest] if hex(timestamp, 8) && hex(text_hash, 16) && hex(digest, 16))
    }
}
