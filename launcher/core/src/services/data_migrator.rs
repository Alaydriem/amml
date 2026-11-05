use crate::{CopyMode, CopyPlan, LauncherError, LauncherPaths, WorldSource};
use amml_gdk::TreeOps;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::SystemTime;

// Copies a `Minecraft Bedrock` folder into the shared data folder. The source is only read.
pub struct DataMigrator {
    appdata_bedrock: PathBuf,
    paths: LauncherPaths,
}

impl DataMigrator {
    const BEDROCK: &'static str = "Minecraft Bedrock";
    const WORLDS: &'static str = "minecraftWorlds";
    const LEVEL_DAT: &'static str = "level.dat";
    const MAX_DEPTH: usize = 8;
    const PART_SUFFIX: &'static str = ".amml-part";

    pub fn new(appdata_bedrock: PathBuf, paths: LauncherPaths) -> DataMigrator {
        DataMigrator { appdata_bedrock, paths }
    }

    // `%APPDATA%\Minecraft Bedrock`, when `APPDATA` is set
    pub fn appdata_bedrock() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|base| PathBuf::from(base).join(Self::BEDROCK))
    }

    // the AppData folder and each instance's own `Minecraft Bedrock` folder, where they exist
    pub fn candidates(&self) -> Vec<WorldSource> {
        let mut found = Vec::new();
        if self.appdata_bedrock.is_dir() {
            found.push(Self::describe(self.appdata_bedrock.clone(), "appdata".to_string()));
        }
        let Ok(entries) = std::fs::read_dir(self.paths.gdk().instances()) else {
            return found;
        };
        let mut folders: Vec<PathBuf> = entries
            .flatten()
            .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
            .map(|entry| entry.path())
            .collect();
        folders.sort();
        for folder in folders {
            let bedrock = folder.join(Self::BEDROCK);
            if bedrock.is_dir() {
                let label = folder.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default();
                found.push(Self::describe(bedrock, label));
            }
        }
        found
    }

    // Decides how to copy. The source and the shared data folder must be separate folders.
    pub fn check(&self, source: &Path, data_dir: &Path) -> Result<CopyMode, LauncherError> {
        let (source_path, data_path) = (Self::resolved(source), Self::resolved(data_dir));
        if data_path.starts_with(&source_path) || source_path.starts_with(&data_path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the source and the shared data folder must be separate folders",
            )
            .into());
        }
        let target = data_dir.join(Self::BEDROCK);
        if target.is_dir() && std::fs::read_dir(&target)?.next().is_some() {
            Ok(CopyMode::Merge)
        } else {
            Ok(CopyMode::Fresh)
        }
    }

    // Lists the files to copy and their sizes. In `Merge` mode a file the target already holds is left out;
    // a `.amml-part` file in the target is ours and does not count as a held file.
    pub fn measure(&self, source: &Path, data_dir: &Path, mode: CopyMode) -> Result<CopyPlan, LauncherError> {
        let target = data_dir.join(Self::BEDROCK);
        let mut files = Vec::new();
        Self::list(source, Path::new(""), &mut files)?;
        if mode == CopyMode::Merge {
            files.retain(|(relative, _)| !target.join(relative).is_file());
        }
        let total_bytes = files.iter().map(|(_, size)| size).sum();
        Ok(CopyPlan { source: source.to_path_buf(), target, mode, files, total_bytes, created: Default::default() })
    }

    // Copies the planned files and returns the bytes copied. `progress(done, total)` runs after each file.
    // `Fresh` goes to a staging folder that is renamed at the end. `Merge` copies each file to a `.amml-part`
    // name and renames it, so a stopped copy never leaves half a file under its real name.
    pub fn copy(&self, plan: &CopyPlan, progress: &mut impl FnMut(u64, u64)) -> Result<u64, LauncherError> {
        match plan.mode {
            CopyMode::Fresh => Self::copy_fresh(plan, progress),
            CopyMode::Merge => Self::copy_merge(plan, progress),
        }
    }

    // Removes the target of a `Fresh` copy, only when that copy made it. A `Merge` target holds the player's
    // own files and stays.
    pub fn undo(&self, plan: &CopyPlan) -> Result<(), LauncherError> {
        if plan.mode == CopyMode::Fresh && plan.created.load(Ordering::SeqCst) && plan.target.exists() {
            TreeOps::remove(&plan.target)?;
        }
        Ok(())
    }

    fn copy_fresh(plan: &CopyPlan, progress: &mut impl FnMut(u64, u64)) -> Result<u64, LauncherError> {
        let Some(data_dir) = plan.target.parent() else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "the target has no parent").into());
        };
        std::fs::create_dir_all(data_dir)?;
        let staging = data_dir.join(format!(".staging-{}", uuid::Uuid::new_v4().simple()));
        let outcome = Self::mirror_dirs(&plan.source, &staging)
            .and_then(|()| Self::copy_files(plan, &staging, false, progress))
            .and_then(|bytes| {
                if plan.target.is_dir() {
                    std::fs::remove_dir(&plan.target)?;
                }
                std::fs::rename(&staging, &plan.target)?;
                plan.created.store(true, Ordering::SeqCst);
                Ok(bytes)
            });
        if outcome.is_err() {
            let _ = TreeOps::remove(&staging);
        }
        outcome
    }

    fn copy_merge(plan: &CopyPlan, progress: &mut impl FnMut(u64, u64)) -> Result<u64, LauncherError> {
        Self::copy_files(plan, &plan.target, true, progress)
    }

    fn copy_files(
        plan: &CopyPlan,
        dest: &Path,
        through_part: bool,
        progress: &mut impl FnMut(u64, u64),
    ) -> Result<u64, LauncherError> {
        let (mut done, mut copied) = (0, 0);
        for (relative, size) in &plan.files {
            let from = plan.source.join(relative);
            let to = dest.join(relative);
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if through_part {
                let part = Self::part_name(&to);
                let published = std::fs::copy(&from, &part).and_then(|_| Self::publish(&part, &to));
                // the part file is ours: it goes away on every outcome
                let _ = std::fs::remove_file(&part);
                if published? {
                    copied += size;
                }
            } else {
                std::fs::copy(&from, &to)?;
                copied += size;
            }
            done += size;
            progress(done, plan.total_bytes);
        }
        Ok(copied)
    }

    fn part_name(path: &Path) -> PathBuf {
        let mut name = path.file_name().map(|name| name.to_os_string()).unwrap_or_default();
        name.push(Self::PART_SUFFIX);
        path.with_file_name(name)
    }

    // Gives `part` the name `to` without replacing a file that appeared there meanwhile. Gives false when
    // the player's file stays. A hard link fails with AlreadyExists at once; a file system without hard links
    // (FAT, exFAT) gets an exists check and a rename, which leaves a small gap.
    fn publish(part: &Path, to: &Path) -> std::io::Result<bool> {
        match std::fs::hard_link(part, to) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(error) if error.kind() == std::io::ErrorKind::Unsupported || Self::no_hard_links(&error) => {
                if to.exists() { Ok(false) } else { std::fs::rename(part, to).map(|()| true) }
            }
            Err(error) => Err(error),
        }
    }

    // ERROR_INVALID_FUNCTION and ERROR_NOT_SUPPORTED
    fn no_hard_links(error: &std::io::Error) -> bool {
        matches!(error.raw_os_error(), Some(1) | Some(50))
    }

    // the path as the file system names it, or as given when it does not exist yet
    fn resolved(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    // every file under `dir`, as a path relative to the first `dir` of the walk, with its size
    fn list(dir: &Path, relative: &Path, files: &mut Vec<(PathBuf, u64)>) -> Result<(), LauncherError> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let next = relative.join(entry.file_name());
            if kind.is_dir() {
                Self::list(&entry.path(), &next, files)?;
            } else if kind.is_file() {
                files.push((next, entry.metadata()?.len()));
            }
        }
        Ok(())
    }

    // creates the folders of `source` under `dest`, empty ones too
    fn mirror_dirs(source: &Path, dest: &Path) -> Result<(), LauncherError> {
        std::fs::create_dir_all(dest)?;
        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                Self::mirror_dirs(&entry.path(), &dest.join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    fn describe(path: PathBuf, label: String) -> WorldSource {
        let mut world_count = 0;
        let mut modified = None;
        Self::scan(&path, 0, &mut world_count, &mut modified);
        WorldSource { path, label, world_count, modified }
    }

    // counts the folders that hold a `level.dat` under any `minecraftWorlds` folder
    fn scan(dir: &Path, depth: usize, count: &mut usize, newest: &mut Option<SystemTime>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                continue;
            }
            if entry.file_name().to_string_lossy().eq_ignore_ascii_case(Self::WORLDS) {
                Self::count_worlds(&entry.path(), count, newest);
            } else if depth < Self::MAX_DEPTH {
                Self::scan(&entry.path(), depth + 1, count, newest);
            }
        }
    }

    fn count_worlds(worlds: &Path, count: &mut usize, newest: &mut Option<SystemTime>) {
        let Ok(entries) = std::fs::read_dir(worlds) else {
            return;
        };
        for world in entries.flatten() {
            let level = world.path().join(Self::LEVEL_DAT);
            let Ok(metadata) = std::fs::metadata(&level) else {
                continue;
            };
            if !metadata.is_file() {
                continue;
            }
            *count += 1;
            if let Ok(modified) = metadata.modified()
                && newest.is_none_or(|held| modified > held)
            {
                *newest = Some(modified);
            }
        }
    }
}
