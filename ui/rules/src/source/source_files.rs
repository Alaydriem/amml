use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The Rust files under one source folder, for a scanner to read.
pub struct SourceFiles {
    root: PathBuf,
    exempt: Vec<String>,
}

impl SourceFiles {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into(), exempt: Vec::new() }
    }

    /// Skips every folder with this name, at any depth (for example "tokens", which holds generated code).
    pub fn exempt_dir(mut self, name: &str) -> Self {
        self.exempt.push(name.to_owned());
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Every `*.rs` file under the root, in sorted order. A root that does not exist is an error, so a scan of a
    /// wrong path cannot pass.
    pub fn rust_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.collect(&self.root, &mut files)?;
        files.sort();
        Ok(files)
    }

    fn collect(&self, dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                if !self.is_exempt(&path) {
                    self.collect(&path, files)?;
                }
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
        Ok(())
    }

    fn is_exempt(&self, dir: &Path) -> bool {
        dir.file_name().is_some_and(|name| self.exempt.iter().any(|exempt| name == exempt.as_str()))
    }
}
