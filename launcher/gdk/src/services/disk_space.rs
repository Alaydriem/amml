use crate::GdkError;
use std::path::Path;
use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::core::HSTRING;

pub struct DiskSpace;

impl DiskSpace {
    pub fn new() -> DiskSpace {
        DiskSpace
    }

    // free bytes available to the current user on the drive that holds `path` (or its nearest existing parent)
    #[allow(unsafe_code)]
    pub fn free_bytes(&self, path: &Path) -> Result<u64, GdkError> {
        let existing = path.ancestors().find(|candidate| candidate.exists()).unwrap_or(path);
        let wide = HSTRING::from(existing.to_string_lossy().as_ref());
        let mut available = 0u64;
        // SAFETY: `wide` is a NUL-terminated UTF-16 string that outlives the call, and `available` is a live u64.
        unsafe { GetDiskFreeSpaceExW(&wide, Some(&mut available), None, None) }
            .map_err(|error| GdkError::Windows { call: "GetDiskFreeSpaceExW", text: error.to_string() })?;
        Ok(available)
    }
}

impl Default for DiskSpace {
    fn default() -> Self {
        Self::new()
    }
}
