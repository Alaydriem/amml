use super::{PeCheck, TreeOps};
use crate::GdkError;
use crate::structs::{AmmlPaths, ExeCopy, GameVersion, VersionFolder};
use std::path::{Path, PathBuf};

pub struct VersionStore {
    paths: AmmlPaths,
}

impl VersionStore {
    pub const EXE_NAME: &'static str = "Minecraft.Windows.exe";
    pub const CONFIG_NAME: &'static str = "MicrosoftGame.Config";
    pub const STAGING_PREFIX: &'static str = ".staging-";

    pub fn new(paths: AmmlPaths) -> VersionStore {
        VersionStore { paths }
    }

    pub fn get(&self, version: &GameVersion) -> Option<VersionFolder> {
        let path = self.paths.versions().join(version.instance_name());
        self.describe(*version, path).ok()
    }

    // complete version folders, newest first; staging folders and folders that are not versions are left out
    pub fn list(&self) -> Result<Vec<VersionFolder>, GdkError> {
        let root = self.paths.versions();
        if !root.exists() {
            return Ok(Vec::new());
        }
        let mut found = Vec::new();
        for entry in std::fs::read_dir(root)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(Self::STAGING_PREFIX) || !entry.file_type()?.is_dir() {
                continue;
            }
            if let Ok(version) = GameVersion::parse(&name) {
                found.push(self.describe(version, entry.path())?);
            }
        }
        found.sort_by_key(|folder| std::cmp::Reverse(folder.version));
        Ok(found)
    }

    // a new empty staging folder inside the version store
    pub fn begin(&self) -> Result<PathBuf, GdkError> {
        let staging = self.paths.versions().join(format!("{}{}", Self::STAGING_PREFIX, uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&staging)?;
        Ok(staging)
    }

    // copies the package files into staging; the file count is returned
    pub fn copy_tree(&self, src: &Path, staging: &Path, skip_exe: bool) -> Result<u64, GdkError> {
        let skip: &[&str] = if skip_exe { &[Self::EXE_NAME] } else { &[] };
        TreeOps::copy(src, staging, skip)
    }

    // The rename at the end is the commit point: a version folder exists only when it is complete.
    // On any failure the staging folder is deleted.
    pub fn commit(
        &self,
        staging: &Path,
        version: &GameVersion,
        exe: ExeCopy,
        expected_files: u64,
    ) -> Result<VersionFolder, GdkError> {
        let result = self.finish(staging, version, &exe, expected_files);
        if result.is_err() {
            let _ = TreeOps::remove(staging);
        }
        result
    }

    // Removes the folder of one version and nothing else. `false` when the version is not in the store.
    pub fn remove(&self, version: &GameVersion) -> Result<bool, GdkError> {
        let path = self.paths.versions().join(version.instance_name());
        if !path.is_dir() {
            return Ok(false);
        }
        TreeOps::remove(&path)?;
        Ok(true)
    }

    // Deletes staging folders left behind by an install that was stopped. A folder that cannot be deleted is
    // logged and left; it must not stop an install.
    pub fn clean_stale(&self) -> Result<Vec<PathBuf>, GdkError> {
        let mut removed = Vec::new();
        for root in [self.paths.versions(), self.paths.instances()] {
            if !root.exists() {
                continue;
            }
            for entry in std::fs::read_dir(root)? {
                let entry = entry?;
                if !(entry.file_name().to_string_lossy().starts_with(Self::STAGING_PREFIX)
                    && entry.file_type()?.is_dir())
                {
                    continue;
                }
                match TreeOps::remove(&entry.path()) {
                    Ok(()) => removed.push(entry.path()),
                    Err(error) => curia::warn!("a stale staging folder was not deleted", {
                        path: entry.path().display().to_string(),
                        error: error.to_string(),
                    }),
                }
            }
        }
        Ok(removed)
    }

    fn finish(
        &self,
        staging: &Path,
        version: &GameVersion,
        exe: &ExeCopy,
        expected_files: u64,
    ) -> Result<VersionFolder, GdkError> {
        let target = self.paths.versions().join(version.instance_name());
        if target.exists() {
            return Err(GdkError::VersionExists(version.instance_name()));
        }
        let staged_exe = staging.join(Self::EXE_NAME);
        if exe.path != staged_exe {
            Self::move_file(&exe.path, &staged_exe)?;
        }
        let (copied, _) = TreeOps::stats(staging)?;
        if copied != expected_files {
            return Err(GdkError::IncompleteCopy { expected: expected_files, copied });
        }
        if !staging.join(Self::CONFIG_NAME).is_file() || !PeCheck::is_x64(&staged_exe)? {
            return Err(GdkError::NotAGame(staging.to_path_buf()));
        }
        std::fs::rename(staging, &target)?;
        self.describe(*version, target)
    }

    fn describe(&self, version: GameVersion, path: PathBuf) -> Result<VersionFolder, GdkError> {
        let (files, bytes) = TreeOps::stats(&path)?;
        Ok(VersionFolder { version, path, files, bytes })
    }

    // a rename, or a copy and delete when the two paths are on different drives
    fn move_file(from: &Path, to: &Path) -> Result<(), GdkError> {
        if std::fs::rename(from, to).is_ok() {
            return Ok(());
        }
        std::fs::copy(from, to)?;
        std::fs::remove_file(from)?;
        Ok(())
    }
}
