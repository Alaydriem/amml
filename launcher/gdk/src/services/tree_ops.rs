use crate::GdkError;
use std::path::Path;

// Folder-level file work shared by the version store and the instance service.
pub struct TreeOps;

impl TreeOps {
    // Copies every file under `src` into `dest`, keeping relative paths. `skip` names top-level entries to leave out.
    // Returns the number of files copied. Read-only flags are cleared so the copy can be deleted later.
    pub fn copy(src: &Path, dest: &Path, skip: &[&str]) -> Result<u64, GdkError> {
        std::fs::create_dir_all(dest)?;
        let mut copied = 0;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let name = entry.file_name();
            if skip.iter().any(|skipped| name.to_string_lossy().eq_ignore_ascii_case(skipped)) {
                continue;
            }
            let target = dest.join(&name);
            if entry.file_type()?.is_dir() {
                copied += Self::copy(&entry.path(), &target, &[])?;
            } else {
                std::fs::copy(entry.path(), &target)?;
                Self::make_writable(&target)?;
                copied += 1;
            }
        }
        Ok(copied)
    }

    // the number of files and their total size under `path`
    pub fn stats(path: &Path) -> Result<(u64, u64), GdkError> {
        let mut files = 0;
        let mut bytes = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let (inner_files, inner_bytes) = Self::stats(&entry.path())?;
                files += inner_files;
                bytes += inner_bytes;
            } else {
                files += 1;
                bytes += entry.metadata()?.len();
            }
        }
        Ok((files, bytes))
    }

    // deletes a folder tree, clearing read-only flags first when the plain delete is refused
    pub fn remove(path: &Path) -> Result<(), GdkError> {
        if !path.exists() {
            return Ok(());
        }
        if std::fs::remove_dir_all(path).is_ok() {
            return Ok(());
        }
        Self::clear_read_only(path)?;
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    fn clear_read_only(path: &Path) -> Result<(), GdkError> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                Self::clear_read_only(&entry.path())?;
            } else {
                Self::make_writable(&entry.path())?;
            }
        }
        Ok(())
    }

    fn make_writable(path: &Path) -> Result<(), GdkError> {
        let mut permissions = std::fs::metadata(path)?.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            std::fs::set_permissions(path, permissions)?;
        }
        Ok(())
    }
}
