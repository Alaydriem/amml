use crate::GdkError;
use std::fs::File;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

// One install at a time. The file is open with no sharing until the lock is dropped.
pub struct InstallLock {
    _file: File,
}

impl InstallLock {
    const SHARING_VIOLATION: i32 = 32;

    pub fn acquire(path: &Path) -> Result<InstallLock, GdkError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::OpenOptions::new().create(true).write(true).truncate(false).share_mode(0).open(path) {
            Ok(file) => Ok(InstallLock { _file: file }),
            Err(error) if error.raw_os_error() == Some(Self::SHARING_VIOLATION) => Err(GdkError::Busy),
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Err(GdkError::Busy),
            Err(error) => Err(error.into()),
        }
    }
}
