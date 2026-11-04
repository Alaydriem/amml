use super::{InstanceGuard, InstanceReader, LauncherBlockStore, ModArchive, ModSwap};
use crate::{LauncherError, LauncherModEntry, ModChange, ModId, ModSource, VerifiedFile};
use amml_gdk::{ProcessInspector, TreeOps};
use amml_registry::ManifestReader;
use std::path::{Path, PathBuf};

// Installs, updates, turns off, turns on and removes the mods of an instance. A failed change leaves the mod folder,
// its `config\` and its `data\` as they were.
pub struct ModInstaller<P: ProcessInspector> {
    guard: InstanceGuard<P>,
}

impl<P: ProcessInspector> ModInstaller<P> {
    pub fn new(guard: InstanceGuard<P>) -> ModInstaller<P> {
        ModInstaller { guard }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn install(
        &self,
        instance: &Path,
        name: &str,
        zip: &VerifiedFile,
        source: ModSource,
        amml_api: u32,
        expected: Option<&ModId>,
        core: Option<bool>,
    ) -> Result<(ModId, ModChange), LauncherError> {
        self.guard.ensure_stopped(instance, name)?;
        let archive = ModArchive::open(&zip.path)?;
        if let Some(wanted) = expected
            && archive.id != *wanted
        {
            return Err(LauncherError::BadArchive {
                file: zip.path.clone(),
                reason: format!("holds {}, not the requested {}", archive.id.as_str(), wanted.as_str()),
            });
        }
        if archive.api_version() != amml_api {
            return Err(LauncherError::ApiMismatch {
                id: archive.id.as_str(),
                mod_api: archive.api_version(),
                amml_api,
            });
        }
        let id = archive.id.clone();
        let mods_root = instance.join("mods");
        let tag = uuid::Uuid::new_v4().simple();
        let staging = mods_root.join(format!(".staging-{tag}"));
        let aside = mods_root.join(format!(".old-{tag}"));
        let enabled_folder = id.folder(&InstanceReader::mods_dir(instance, true));
        let disabled_folder = id.folder(&InstanceReader::mods_dir(instance, false));
        let existing = [&enabled_folder, &disabled_folder].into_iter().find(|folder| folder.is_dir()).cloned();
        let from = existing.as_ref().and_then(|folder| Self::version_of(folder));
        let entry = LauncherModEntry {
            version: Some(archive.manifest.version.clone()).filter(|version| !version.is_empty()),
            source,
            core,
        };

        let mut swap = ModSwap::new(staging.clone(), aside, existing.clone());
        let outcome = archive
            .extract_to(&staging)
            .and_then(|()| swap.run(existing.as_ref().unwrap_or(&enabled_folder)))
            .and_then(|()| LauncherBlockStore::set_mod(instance, &id.as_str(), entry));
        match outcome {
            Ok(()) => {
                swap.finish();
                let change = if existing.is_some() { ModChange::Updated { from } } else { ModChange::Installed };
                Ok((id, change))
            }
            Err(error) => {
                swap.undo(existing.as_ref().unwrap_or(&enabled_folder));
                Err(error)
            }
        }
    }

    // moves a mod between `mods\amml` and `mods\amml-disabled`; a core mod cannot be turned off
    pub fn set_enabled(
        &self,
        instance: &Path,
        name: &str,
        id: &ModId,
        enabled: bool,
        core: bool,
    ) -> Result<(), LauncherError> {
        if core && !enabled {
            return Err(LauncherError::CoreMod(id.as_str()));
        }
        self.guard.ensure_stopped(instance, name)?;
        let from = id.folder(&InstanceReader::mods_dir(instance, !enabled));
        let to = id.folder(&InstanceReader::mods_dir(instance, enabled));
        if !from.is_dir() {
            return if to.is_dir() { Ok(()) } else { Err(LauncherError::UnknownMod(id.as_str())) };
        }
        if to.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("{} exists already", to.display()),
            )
            .into());
        }
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&from, &to)?;
        Self::drop_empty_parent(&from);
        Ok(())
    }

    pub fn remove(&self, instance: &Path, name: &str, id: &ModId, core: bool) -> Result<(), LauncherError> {
        if core {
            return Err(LauncherError::CoreMod(id.as_str()));
        }
        self.guard.ensure_stopped(instance, name)?;
        let folders = [true, false].map(|enabled| id.folder(&InstanceReader::mods_dir(instance, enabled)));
        let present: Vec<&PathBuf> = folders.iter().filter(|folder| folder.is_dir()).collect();
        if present.is_empty() {
            return Err(LauncherError::UnknownMod(id.as_str()));
        }
        let tag = uuid::Uuid::new_v4().simple();
        let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut outcome = Ok(());
        for (index, folder) in present.into_iter().enumerate() {
            let aside = instance.join("mods").join(format!(".old-{tag}-{index}"));
            match std::fs::rename(folder, &aside) {
                Ok(()) => moved.push((folder.clone(), aside)),
                Err(error) => {
                    outcome = Err(error.into());
                    break;
                }
            }
        }
        if outcome.is_ok() && instance.join("version.json").is_file() {
            outcome = LauncherBlockStore::remove_mod(instance, &id.as_str());
        }
        if outcome.is_err() {
            for (folder, aside) in moved.iter().rev() {
                if let Err(error) = std::fs::rename(aside, folder) {
                    curia::error!("a mod folder was not put back", {
                        folder: folder.display().to_string(),
                        error: error.to_string(),
                    });
                }
            }
            return outcome;
        }
        for (folder, aside) in moved {
            if let Err(error) = TreeOps::remove(&aside) {
                curia::warn!("a removed mod folder was not deleted", {
                    folder: aside.display().to_string(),
                    error: error.to_string(),
                });
            }
            Self::drop_empty_parent(&folder);
        }
        Ok(())
    }

    fn version_of(folder: &Path) -> Option<String> {
        let text = std::fs::read_to_string(folder.join("amml.hcl")).ok()?;
        let manifest = ManifestReader::parse(&text).ok()?;
        Some(manifest.version).filter(|version| !version.is_empty())
    }

    // removes `@namespace` when the last mod left it
    fn drop_empty_parent(folder: &Path) {
        if let Some(parent) = folder.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }
}
