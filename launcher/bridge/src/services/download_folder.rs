use super::UnixMillis;
use crate::{BridgeError, DownloadFileStateView, DownloadFileView, DownloadsView};
use std::path::{Path, PathBuf};

// The downloads folder: what is in it, and what the player may remove from it.
pub struct DownloadFolder {
    dir: PathBuf,
    // file names a restore needs: the package of the Store version that is registered now
    kept: Vec<String>,
}

impl DownloadFolder {
    const PART: &'static str = ".part";
    const PART_META: &'static str = ".part.meta";

    pub fn new(dir: PathBuf, kept: Vec<String>) -> DownloadFolder {
        DownloadFolder { dir, kept }
    }

    pub fn view(&self) -> DownloadsView {
        let files = self.files();
        DownloadsView {
            folder: self.dir.display().to_string(),
            used: files.iter().map(|file| file.size).sum(),
            files: files.into_iter().filter(|file| !file.id.ends_with(Self::PART_META)).collect(),
        }
    }

    // Deletes one file. A name with a separator, a `..` or a file that must stay is refused.
    pub fn remove(&self, id: &str) -> Result<(), BridgeError> {
        if id.is_empty() || id.contains(['/', '\\']) || id.contains("..") || self.is_kept(id) {
            return Err(BridgeError::Refused(id.to_string()));
        }
        let path = self.dir.join(id);
        if !path.is_file() {
            return Err(BridgeError::Unavailable(id.to_string()));
        }
        std::fs::remove_file(&path)?;
        if let Some(meta) = Self::meta_of(&path) {
            let _ = std::fs::remove_file(meta);
        }
        Ok(())
    }

    // Deletes every partial download with its meta file; returns how many partial files went.
    pub fn remove_unfinished(&self) -> Result<u32, BridgeError> {
        let mut removed = 0;
        for file in self.files() {
            if file.id.ends_with(Self::PART_META) {
                std::fs::remove_file(self.dir.join(&file.id))?;
            } else if file.state == DownloadFileStateView::Unfinished {
                std::fs::remove_file(self.dir.join(&file.id))?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    // File names on Windows ignore case, so a kept name matches a file whatever its case
    fn is_kept(&self, id: &str) -> bool {
        self.kept.iter().any(|kept| kept.eq_ignore_ascii_case(id))
    }

    fn files(&self) -> Vec<DownloadFileView> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut files: Vec<DownloadFileView> = entries
            .flatten()
            .filter_map(|entry| {
                let meta = entry.metadata().ok().filter(|meta| meta.is_file())?;
                let id = entry.file_name().to_string_lossy().to_string();
                let state = if self.is_kept(&id) {
                    DownloadFileStateView::Kept
                } else if id.ends_with(Self::PART) || id.ends_with(Self::PART_META) {
                    DownloadFileStateView::Unfinished
                } else {
                    DownloadFileStateView::Finished
                };
                let modified = meta.modified().map(UnixMillis::of).unwrap_or(0);
                Some(DownloadFileView { id, size: meta.len(), state, modified })
            })
            .collect();
        files.sort_by(|left, right| left.id.cmp(&right.id));
        files
    }

    // the `.part.meta` file that belongs to a partial download
    fn meta_of(path: &Path) -> Option<PathBuf> {
        let name = path.file_name()?.to_string_lossy().to_string();
        name.ends_with(Self::PART).then(|| path.with_file_name(format!("{name}.meta")))
    }
}
