use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// A temporary source folder that a test fills with files.
pub struct SourceTree {
    dir: TempDir,
}

impl SourceTree {
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap_or_else(|error| panic!("temp dir: {error}"));
        Self { dir }
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    /// Writes `text` to `relative` (for example "widget/mod.rs"), with its folders.
    pub fn file(&self, relative: &str, text: &str) -> PathBuf {
        let path = self.dir.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| panic!("create {}: {error}", parent.display()));
        }
        fs::write(&path, text).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
        path
    }

    pub fn folder(&self, relative: &str) {
        let path = self.dir.path().join(relative);
        fs::create_dir_all(&path).unwrap_or_else(|error| panic!("create {}: {error}", path.display()));
    }
}
