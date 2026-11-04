use crate::LauncherError;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

// The rules every archive the launcher unpacks must follow, and the one way it unpacks them.
pub struct ZipGuard;

impl ZipGuard {
    pub const MAX_ENTRIES: usize = 20_000;
    pub const MAX_UNPACKED: u64 = 512 * 1024 * 1024;
    const COPY_BLOCK: usize = 64 * 1024;

    // Every entry as (index, relative path). An entry that could leave the target folder, or an archive that is
    // too big, is refused.
    pub fn entries(archive: &mut ZipArchive<File>) -> Result<Vec<(usize, PathBuf)>, LauncherError> {
        if archive.len() > Self::MAX_ENTRIES {
            return Err(Self::bad(PathBuf::new(), format!("has more than {} entries", Self::MAX_ENTRIES)));
        }
        let mut found = Vec::with_capacity(archive.len());
        let mut unpacked: u64 = 0;
        for index in 0..archive.len() {
            let entry = archive.by_index_raw(index).map_err(|error| Self::bad(PathBuf::new(), error.to_string()))?;
            let name = entry.name().map_err(|error| Self::bad(PathBuf::new(), error.to_string()))?.to_string();
            let path = Self::relative(&name)?;
            if entry.is_symlink() {
                return Err(Self::bad(path, "is a symbolic link".to_string()));
            }
            unpacked = unpacked.saturating_add(entry.size());
            if unpacked > Self::MAX_UNPACKED {
                return Err(Self::bad(PathBuf::new(), format!("unpacks to more than {} bytes", Self::MAX_UNPACKED)));
            }
            found.push((index, path));
        }
        Ok(found)
    }

    // Writes the entries below `prefix` into `dest`, without the prefix. The bytes written are counted, not trusted.
    pub fn extract(
        archive: &mut ZipArchive<File>,
        entries: &[(usize, PathBuf)],
        prefix: &Path,
        dest: &Path,
    ) -> Result<(), LauncherError> {
        std::fs::create_dir_all(dest)?;
        let mut unpacked: u64 = 0;
        for (index, path) in entries {
            let Ok(relative) = path.strip_prefix(prefix) else {
                continue;
            };
            if relative.as_os_str().is_empty() {
                continue;
            }
            let target = dest.join(relative);
            let mut entry = archive.by_index(*index).map_err(|error| Self::bad(path.clone(), error.to_string()))?;
            if entry.is_dir() {
                std::fs::create_dir_all(&target)?;
                continue;
            }
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let declared = entry.size();
            let mut out = File::create(&target)?;
            let mut written: u64 = 0;
            let mut block = vec![0u8; Self::COPY_BLOCK];
            loop {
                let read = entry.read(&mut block).map_err(|error| Self::bad(path.clone(), error.to_string()))?;
                if read == 0 {
                    break;
                }
                written += read as u64;
                unpacked += read as u64;
                if written > declared || unpacked > Self::MAX_UNPACKED {
                    return Err(Self::bad(path.clone(), "is larger than the archive says".to_string()));
                }
                out.write_all(&block[..read])?;
            }
            out.flush()?;
        }
        Ok(())
    }

    // Reads one entry into memory. The entry may not be larger than `limit`.
    pub fn read_entry(archive: &mut ZipArchive<File>, index: usize, limit: u64) -> Result<Vec<u8>, LauncherError> {
        let mut entry = archive.by_index(index).map_err(|error| Self::bad(PathBuf::new(), error.to_string()))?;
        let name = PathBuf::from(entry.name().map(|name| name.to_string()).unwrap_or_default());
        if entry.size() > limit {
            return Err(Self::bad(name, format!("is larger than {limit} bytes")));
        }
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| Self::bad(name.clone(), error.to_string()))?;
        if bytes.len() as u64 > limit {
            return Err(Self::bad(name, format!("is larger than {limit} bytes")));
        }
        Ok(bytes)
    }

    pub fn bad(file: PathBuf, reason: String) -> LauncherError {
        LauncherError::BadArchive { file, reason }
    }

    // the path of an entry, or an error when the name is anything but a plain relative path
    fn relative(name: &str) -> Result<PathBuf, LauncherError> {
        let refuse = |reason: &str| Self::bad(PathBuf::from(name), reason.to_string());
        if name.contains('\\') {
            return Err(refuse("uses a backslash"));
        }
        if name.starts_with('/') {
            return Err(refuse("is an absolute path"));
        }
        if name.chars().any(char::is_control) {
            return Err(refuse("has a control character"));
        }
        let trimmed = name.strip_suffix('/').unwrap_or(name);
        let mut path = PathBuf::new();
        for part in trimmed.split('/') {
            if part.is_empty() || part == "." || part == ".." {
                return Err(refuse("leaves its folder"));
            }
            if part.contains(':') {
                return Err(refuse("has a drive prefix"));
            }
            path.push(part);
        }
        Ok(path)
    }
}
