use super::{InstanceHclFile, VersionJsonFile};
use crate::{AmmlState, InstalledMod, InstanceState, LauncherBlock, LauncherError, LauncherPaths, ModId, ModSource};
use amml_registry::ManifestReader;
use std::path::{Path, PathBuf};

// Reads what is on disk in an instance folder.
pub struct InstanceReader;

impl InstanceReader {
    pub const EXE_NAME: &'static str = "Minecraft.Windows.exe";
    pub const BACKUP_NAME: &'static str = "Minecraft.Windows.exe.bak";
    pub const VERSION_JSON: &'static str = "version.json";
    const ENABLED_MODS: [&'static str; 2] = ["mods", "amml"];
    const DISABLED_MODS: [&'static str; 2] = ["mods", "amml-disabled"];

    pub fn read(folder: &Path) -> Result<InstanceState, LauncherError> {
        let version_path = folder.join(Self::VERSION_JSON);
        if !version_path.is_file() || !folder.join(Self::BACKUP_NAME).is_file() {
            return Err(LauncherError::NotAnInstance(folder.to_path_buf()));
        }
        let version_json = VersionJsonFile::read(&version_path)?;
        let launcher = version_json.launcher();
        let name = folder.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default();
        Ok(InstanceState {
            name,
            folder: folder.to_path_buf(),
            game: version_json.game_version()?,
            preview: version_json.preview(),
            amml: Self::amml(folder),
            mods: Self::mods(folder, launcher.as_ref()),
            data_dir: InstanceHclFile::read_data_dir(&Self::instance_hcl(folder), folder),
            managed: launcher.is_some(),
        })
    }

    // every folder below `instances\` that does not start with a dot
    pub fn list(paths: &LauncherPaths) -> Vec<Result<InstanceState, LauncherError>> {
        let Ok(entries) = std::fs::read_dir(paths.gdk().instances()) else {
            return Vec::new();
        };
        let mut folders: Vec<PathBuf> = entries
            .flatten()
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
            .map(|entry| entry.path())
            .collect();
        folders.sort();
        folders.iter().map(|folder| Self::read(folder)).collect()
    }

    pub fn instance_hcl(folder: &Path) -> PathBuf {
        folder.join("amml").join("instance.hcl")
    }

    pub fn mods_dir(folder: &Path, enabled: bool) -> PathBuf {
        let parts = if enabled { Self::ENABLED_MODS } else { Self::DISABLED_MODS };
        parts.iter().fold(folder.to_path_buf(), |path, part| path.join(part))
    }

    fn amml(folder: &Path) -> AmmlState {
        let installed = folder.join("amml.dll").is_file();
        let version = if installed {
            std::fs::read_to_string(folder.join("amml").join("amml-version.txt"))
                .ok()
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
        } else {
            None
        };
        AmmlState { version, installed }
    }

    fn mods(folder: &Path, launcher: Option<&LauncherBlock>) -> Vec<InstalledMod> {
        let mut found = Vec::new();
        for enabled in [true, false] {
            let root = Self::mods_dir(folder, enabled);
            let Ok(namespaces) = std::fs::read_dir(&root) else {
                continue;
            };
            for namespace in namespaces.flatten() {
                let namespace_name = namespace.file_name().to_string_lossy().to_string();
                if !namespace_name.starts_with('@') || !namespace.file_type().map(|kind| kind.is_dir()).unwrap_or(false)
                {
                    continue;
                }
                let Ok(names) = std::fs::read_dir(namespace.path()) else {
                    continue;
                };
                for name in names.flatten() {
                    if !name.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                        continue;
                    }
                    let raw = format!("{namespace_name}/{}", name.file_name().to_string_lossy());
                    match ModId::parse(&raw) {
                        Ok(id) => found.push(Self::installed(id, name.path(), enabled, launcher)),
                        Err(error) => curia::warn!("a mod folder has no valid id", {
                            folder: name.path().display().to_string(),
                            error: error.to_string(),
                        }),
                    }
                }
            }
        }
        found.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| b.enabled.cmp(&a.enabled)));
        found
    }

    fn installed(id: ModId, folder: PathBuf, enabled: bool, launcher: Option<&LauncherBlock>) -> InstalledMod {
        let source = launcher
            .and_then(|block| block.mods.get(&id.as_str()))
            .map(|entry| entry.source.clone())
            .unwrap_or(ModSource::Local);
        let manifest = std::fs::read_to_string(folder.join("amml.hcl"))
            .map_err(|error| error.to_string())
            .and_then(|text| ManifestReader::parse(&text).map_err(|error| error.to_string()));
        match manifest {
            Ok(manifest) => InstalledMod {
                id,
                version: Some(manifest.version).filter(|version| !version.is_empty()),
                enabled,
                listing: manifest.listing,
                api_version: manifest.api_version,
                source,
                folder,
            },
            Err(error) => {
                curia::warn!("a mod has no readable amml.hcl", {
                    folder: folder.display().to_string(),
                    error: error,
                });
                InstalledMod { id, version: None, enabled, listing: None, api_version: 0, source, folder }
            }
        }
    }
}
