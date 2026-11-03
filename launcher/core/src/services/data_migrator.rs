use crate::{LauncherError, LauncherPaths, WorldSource};
use amml_gdk::TreeOps;
use std::path::{Path, PathBuf};
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

    // Copies the contents of `source` to `<data_dir>\Minecraft Bedrock\` and returns the number of files.
    // The target must be missing or empty. The copy goes to a staging folder first and is renamed at the end.
    pub fn copy(&self, source: &Path, data_dir: &Path, progress: &mut impl FnMut(u64)) -> Result<u64, LauncherError> {
        let target = data_dir.join(Self::BEDROCK);
        let (source_path, data_path) = (Self::resolved(source), Self::resolved(data_dir));
        if data_path.starts_with(&source_path) || source_path.starts_with(&data_path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the source and the shared data folder must be separate folders",
            )
            .into());
        }
        if target.is_dir() && std::fs::read_dir(&target)?.next().is_some() {
            return Err(LauncherError::TargetNotEmpty(target));
        }
        std::fs::create_dir_all(data_dir)?;
        let staging = data_dir.join(format!(".staging-{}", uuid::Uuid::new_v4().simple()));
        let outcome = Self::copy_tree(source, &staging, &mut 0, progress).and_then(|count| {
            if target.is_dir() {
                std::fs::remove_dir(&target)?;
            }
            std::fs::rename(&staging, &target)?;
            Ok(count)
        });
        if outcome.is_err() {
            let _ = TreeOps::remove(&staging);
        }
        outcome
    }

    // the path as the file system names it, or as given when it does not exist yet
    fn resolved(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    fn copy_tree(
        source: &Path,
        dest: &Path,
        count: &mut u64,
        progress: &mut impl FnMut(u64),
    ) -> Result<u64, LauncherError> {
        std::fs::create_dir_all(dest)?;
        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let target = dest.join(entry.file_name());
            if kind.is_dir() {
                Self::copy_tree(&entry.path(), &target, count, progress)?;
            } else if kind.is_file() {
                std::fs::copy(entry.path(), &target)?;
                *count += 1;
                progress(*count);
            }
        }
        Ok(*count)
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
