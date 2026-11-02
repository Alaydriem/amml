use crate::GdkError;
use std::path::{Path, PathBuf};

pub struct DownloadCache {
    dir: PathBuf,
}

impl DownloadCache {
    const EXTENSIONS: [&'static str; 3] = [".msixvc", ".part", ".part.meta"];

    pub fn new(dir: PathBuf) -> DownloadCache {
        DownloadCache { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    // a complete file of exactly `size` bytes
    pub fn cached(&self, file_name: &str, size: u64) -> Option<PathBuf> {
        let path = self.dir.join(file_name);
        let meta = std::fs::metadata(&path).ok()?;
        if meta.is_file() && meta.len() == size { Some(path) } else { None }
    }

    // deletes every package file and partial download that is not named in `keep`
    pub fn retain_only(&self, keep: &[&str]) -> Result<Vec<PathBuf>, GdkError> {
        let mut deleted = Vec::new();
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(deleted),
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            let is_cache_file = Self::EXTENSIONS.iter().any(|extension| name.ends_with(extension));
            if !is_cache_file || keep.contains(&name.as_str()) {
                continue;
            }
            std::fs::remove_file(entry.path())?;
            deleted.push(entry.path());
        }
        Ok(deleted)
    }
}
