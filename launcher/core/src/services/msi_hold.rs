use crate::LauncherError;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

// An installer file held open for reading only: while it lives, nothing can write to the file, replace it or delete
// it, so what was hashed is what runs.
pub struct MsiHold {
    file: File,
}

impl MsiHold {
    const BLOCK: usize = 64 * 1024;
    // FILE_SHARE_READ: others may read, none may write or delete
    const SHARE_READ: u32 = 0x1;

    pub fn open(path: &Path) -> Result<MsiHold, LauncherError> {
        let file = OpenOptions::new().read(true).share_mode(Self::SHARE_READ).open(path)?;
        Ok(MsiHold { file })
    }

    // lowercase hex of the whole file, read through the held handle
    pub fn sha256(&mut self) -> Result<String, LauncherError> {
        use std::io::Seek;
        self.file.rewind()?;
        let mut hasher = Sha256::new();
        let mut block = vec![0u8; Self::BLOCK];
        loop {
            let read = self.file.read(&mut block)?;
            if read == 0 {
                break;
            }
            hasher.update(&block[..read]);
        }
        Ok(hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
    }
}
