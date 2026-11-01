use std::fs;
use std::path::{Path, PathBuf};

use amml_data::DataError;
use amml_derive::DERIVER_VERSION;
use xxhash_rust::xxh3::Xxh3;

/// A hash of the recipe files, so a cache made from other recipes is never used.
pub struct RecipeDigest;

impl RecipeDigest {
    /// xxh3-64 as 16 hex digits over `DERIVER_VERSION` and every `.hcl` file under `dir`: relative
    /// path and contents, in sorted path order.
    pub fn of_dir(dir: &Path) -> Result<String, DataError> {
        Self::of_dir_with(dir, DERIVER_VERSION)
    }

    /// `of_dir` for a given deriver version.
    pub fn of_dir_with(dir: &Path, deriver_version: u32) -> Result<String, DataError> {
        let mut files = Vec::new();
        Self::collect(dir, &mut files)?;
        files.sort();
        let mut hasher = Xxh3::new();
        hasher.update(&deriver_version.to_le_bytes());
        for path in &files {
            let bytes = fs::read(path).map_err(|source| DataError::Io { path: path.clone(), source })?;
            let relative = path.strip_prefix(dir).unwrap_or(path);
            let parts: Vec<_> = relative.components().map(|part| part.as_os_str().to_string_lossy()).collect();
            let name = parts.join("/");
            hasher.update(&(name.len() as u64).to_le_bytes());
            hasher.update(name.as_bytes());
            hasher.update(&(bytes.len() as u64).to_le_bytes());
            hasher.update(&bytes);
        }
        Ok(format!("{:016x}", hasher.digest()))
    }

    fn collect(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), DataError> {
        let io = |source| DataError::Io { path: dir.to_path_buf(), source };
        for item in fs::read_dir(dir).map_err(io)? {
            let path = item.map_err(io)?.path();
            if path.is_dir() {
                Self::collect(&path, files)?;
            } else if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("hcl")) {
                files.push(path);
            }
        }
        Ok(())
    }
}
