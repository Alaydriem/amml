use super::{InstallLock, TreeOps, VersionStore};
use crate::GdkError;
use crate::structs::{AmmlPaths, ExeCopy, GameVersion, InstanceFolder, VersionFolder};
use std::path::Path;

// Makes instances from the version store and seeds the store from an instance that already exists.
// Neither needs the Store, the network or administrator rights.
pub struct InstanceService {
    paths: AmmlPaths,
    store: VersionStore,
}

impl InstanceService {
    const BACKUP_NAME: &'static str = "Minecraft.Windows.exe.bak";
    const NAME_LIMIT: usize = 64;
    // top-level entries of an instance that belong to the player or to AMML, not to the game version
    const INSTANCE_ONLY: [&'static str; 8] = [
        "amml",
        "mods",
        "logs",
        "Minecraft Bedrock",
        "version.json",
        "amml.dll",
        VersionStore::EXE_NAME,
        InstanceService::BACKUP_NAME,
    ];

    pub fn new(paths: AmmlPaths, store: VersionStore) -> InstanceService {
        InstanceService { paths, store }
    }

    // A copy of a stored version in `instances\<name>`, with the original exe kept as `Minecraft.Windows.exe.bak`.
    pub fn create(&self, version: &GameVersion, name: &str) -> Result<InstanceFolder, GdkError> {
        Self::check_name(name)?;
        // an install deletes stale staging folders; it must not delete this one
        let _lock = InstallLock::acquire(&self.paths.lock())?;
        let target = self.paths.instances().join(name);
        if target.exists() {
            return Err(GdkError::InstanceExists(name.to_string()));
        }
        let folder = self.store.get(version).ok_or_else(|| GdkError::UnknownVersion(version.to_string()))?;
        let staging =
            self.paths.instances().join(format!("{}{}", VersionStore::STAGING_PREFIX, uuid::Uuid::new_v4().simple()));
        let result = self.build(&folder, &staging, &target);
        if result.is_err() {
            let _ = TreeOps::remove(&staging);
        }
        result?;
        Ok(InstanceFolder { name: name.to_string(), version: *version, path: target })
    }

    // Reads an instance and stores its game files as `version`. The instance is not changed.
    // The unpatched exe is the instance's `.bak`.
    pub fn seed_version_from_instance(
        &self,
        instance_dir: &Path,
        version: &GameVersion,
    ) -> Result<VersionFolder, GdkError> {
        let _lock = InstallLock::acquire(&self.paths.lock())?;
        let backup = instance_dir.join(Self::BACKUP_NAME);
        if !backup.is_file() {
            return Err(GdkError::NotAGame(instance_dir.to_path_buf()));
        }
        if self.store.get(version).is_some() {
            return Err(GdkError::VersionExists(version.instance_name()));
        }
        let staging = self.store.begin()?;
        let copied = match TreeOps::copy(instance_dir, &staging, &Self::INSTANCE_ONLY) {
            Ok(copied) => copied,
            Err(error) => {
                let _ = TreeOps::remove(&staging);
                return Err(error);
            }
        };
        let exe_path = staging.join(VersionStore::EXE_NAME);
        let placed = std::fs::copy(&backup, &exe_path).map_err(GdkError::from);
        let bytes = match placed {
            Ok(bytes) => bytes,
            Err(error) => {
                let _ = TreeOps::remove(&staging);
                return Err(error);
            }
        };
        let exe = ExeCopy { path: exe_path, bytes, method: "seed" };
        self.store.commit(&staging, version, exe, copied + 1)
    }

    fn build(&self, folder: &VersionFolder, staging: &Path, target: &Path) -> Result<(), GdkError> {
        TreeOps::copy(&folder.path, staging, &[])?;
        std::fs::copy(folder.path.join(VersionStore::EXE_NAME), staging.join(Self::BACKUP_NAME))?;
        std::fs::rename(staging, target)?;
        Ok(())
    }

    // Windows device names are not file names, with or without an extension: `CON`, `NUL.txt`, `COM1`
    fn is_device_name(name: &str) -> bool {
        let stem = name.split('.').next().unwrap_or_default().to_ascii_uppercase();
        let numbered = |prefix: &str| {
            stem.strip_prefix(prefix).is_some_and(|rest| rest.len() == 1 && matches!(rest.as_bytes()[0], b'1'..=b'9'))
        };
        matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") || numbered("COM") || numbered("LPT")
    }

    // `[A-Za-z0-9._-]{1,64}`, not `.` or `..`, no trailing dot, no device name, and not a staging name
    pub fn check_name(name: &str) -> Result<(), GdkError> {
        let allowed = |character: char| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-');
        let valid = !name.is_empty()
            && name.len() <= Self::NAME_LIMIT
            && name.chars().all(allowed)
            && name != "."
            && name != ".."
            && !name.ends_with('.')
            && !Self::is_device_name(name)
            && !name.starts_with(VersionStore::STAGING_PREFIX);
        if valid { Ok(()) } else { Err(GdkError::BadInstanceName(name.to_string())) }
    }
}
