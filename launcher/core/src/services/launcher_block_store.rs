use super::VersionJsonFile;
use crate::{LauncherBlock, LauncherError, LauncherModEntry};
use std::path::Path;

// Read-modify-write of the `launcher` block in an instance's `version.json`. Every other key stays as it is.
pub struct LauncherBlockStore;

impl LauncherBlockStore {
    const FILE: &'static str = "version.json";

    pub fn read(instance: &Path) -> Result<LauncherBlock, LauncherError> {
        let file = VersionJsonFile::read(&instance.join(Self::FILE))?;
        Ok(file.launcher().unwrap_or_default())
    }

    pub fn set_mod(instance: &Path, id: &str, entry: LauncherModEntry) -> Result<(), LauncherError> {
        Self::update(instance, |block| {
            block.mods.insert(id.to_string(), entry);
        })
    }

    pub fn remove_mod(instance: &Path, id: &str) -> Result<(), LauncherError> {
        Self::update(instance, |block| {
            block.mods.remove(id);
        })
    }

    pub fn set_amml(instance: &Path, version: &str) -> Result<(), LauncherError> {
        Self::update(instance, |block| {
            block.amml = Some(version.to_string());
        })
    }

    fn update(instance: &Path, change: impl FnOnce(&mut LauncherBlock)) -> Result<(), LauncherError> {
        let path = instance.join(Self::FILE);
        let mut file = VersionJsonFile::read(&path)?;
        let mut block = file.launcher().unwrap_or_default();
        change(&mut block);
        file.set_launcher(&block);
        file.write(&path)
    }
}
