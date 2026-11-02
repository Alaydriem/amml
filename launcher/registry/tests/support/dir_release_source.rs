use std::fs;
use std::path::PathBuf;

use amml_registry::{HashText, RegistryError, ReleaseSource};

/// Serves release assets from a folder laid out as `<root>/<repo>/<tag>/<asset>`.
pub struct DirReleaseSource {
    root: PathBuf,
}

impl DirReleaseSource {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn put(&self, repo: &str, tag: &str, asset: &str, bytes: &[u8]) {
        let dir = self.root.join(repo).join(tag);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(asset), bytes).unwrap();
    }

    /// Stores a zip with a matching `SHA256SUMS.txt`.
    pub fn put_release(&self, repo: &str, tag: &str, asset: &str, zip: &[u8]) {
        self.put(repo, tag, asset, zip);
        let sums = format!("{}  {asset}\n", HashText::sha256(zip));
        self.put(repo, tag, "SHA256SUMS.txt", sums.as_bytes());
    }
}

impl ReleaseSource for DirReleaseSource {
    fn fetch(&self, repo: &str, tag: &str, asset: &str) -> Result<Vec<u8>, RegistryError> {
        fs::read(self.root.join(repo).join(tag).join(asset)).map_err(|_| RegistryError::Missing(asset.to_string()))
    }
}
