use crate::LauncherError;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub struct FileHash;

impl FileHash {
    const BLOCK: usize = 64 * 1024;

    // lowercase hex, read in 64 KB blocks
    pub fn sha256(path: &Path) -> Result<String, LauncherError> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut block = vec![0u8; Self::BLOCK];
        loop {
            let read = file.read(&mut block)?;
            if read == 0 {
                break;
            }
            hasher.update(&block[..read]);
        }
        Ok(hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
    }
}
