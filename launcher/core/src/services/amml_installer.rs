use super::{AmmlArchive, InstanceGuard, Patcher};
use crate::{CoreProgress, LauncherError, PlacedItem, VerifiedFile};
use amml_gdk::{ProcessInspector, TreeOps};
use std::path::{Path, PathBuf};

// Puts AMML into an instance: the loader, its recipes, its solved data (when the release has any), its version and the patched game executable.
// A failure leaves the instance as it was.
pub struct AmmlInstaller<T: Patcher, P: ProcessInspector> {
    patcher: T,
    guard: InstanceGuard<P>,
}

impl<T: Patcher, P: ProcessInspector> AmmlInstaller<T, P> {
    const BACKUP_NAME: &'static str = "Minecraft.Windows.exe.bak";
    const EXE_NAME: &'static str = "Minecraft.Windows.exe";

    pub fn new(patcher: T, guard: InstanceGuard<P>) -> AmmlInstaller<T, P> {
        AmmlInstaller { patcher, guard }
    }

    // installs the zip into the instance and returns the AMML version
    pub fn install(
        &self,
        instance: &Path,
        name: &str,
        zip: &VerifiedFile,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<String, LauncherError> {
        self.guard.ensure_stopped(instance, name)?;
        let archive = AmmlArchive::open(&zip.path)?;
        let version = archive.version();
        let backup = instance.join(Self::BACKUP_NAME);
        if !backup.is_file() {
            return Err(LauncherError::NotAnInstance(instance.to_path_buf()));
        }
        progress(CoreProgress::InstallingAmml { version: version.clone() });
        let staging = instance.join(format!(".amml-staging-{}", uuid::Uuid::new_v4().simple()));
        let outcome = self.stage_and_place(&archive, instance, &staging, &backup, progress);
        let _ = TreeOps::remove(&staging);
        outcome.map(|()| version)
    }

    fn stage_and_place(
        &self,
        archive: &AmmlArchive,
        instance: &Path,
        staging: &Path,
        backup: &Path,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<(), LauncherError> {
        archive.extract_to(staging)?;
        progress(CoreProgress::Patching);
        let patched = staging.join(Self::EXE_NAME);
        self.patcher.patch(&staging.join("amml-patch.exe"), backup, &patched)?;
        if !patched.is_file() {
            return Err(LauncherError::PatchFailed { code: Some(0), text: "the patcher wrote no file".to_string() });
        }
        let mut moves = vec![
            (staging.join("amml.dll"), instance.join("amml.dll")),
            (staging.join("amml").join("recipes"), instance.join("amml").join("recipes")),
            (staging.join("amml").join("amml-version.txt"), instance.join("amml").join("amml-version.txt")),
            (patched, instance.join(Self::EXE_NAME)),
        ];
        let staged_data = staging.join("amml").join("data");
        let instance_data = instance.join("amml").join("data");
        let mut placed: Vec<PlacedItem> = Vec::new();
        let mut created: Vec<PathBuf> = Vec::new();
        // a release without solved data must not leave the old release's data behind
        let drop_old_data = !staged_data.exists() && instance_data.exists();
        if staged_data.exists() {
            moves.push((staged_data, instance_data.clone()));
        }
        let result = Self::drop_data(drop_old_data, &instance_data, &mut placed)
            .and_then(|()| Self::place_all(instance, &moves, &mut placed, &mut created, progress));
        match result {
            Ok(()) => {
                for item in &placed {
                    if let Some(old) = &item.replaced {
                        Self::delete(old);
                    }
                }
                Ok(())
            }
            Err(error) => {
                Self::undo(&placed, &created);
                Err(error)
            }
        }
    }

    // moves the old solved data aside so a failed install can put it back
    fn drop_data(wanted: bool, data: &Path, placed: &mut Vec<PlacedItem>) -> Result<(), LauncherError> {
        if !wanted {
            return Ok(());
        }
        let aside = Self::aside(data);
        std::fs::rename(data, &aside)?;
        placed.push(PlacedItem { target: data.to_path_buf(), replaced: Some(aside) });
        Ok(())
    }

    fn place_all(
        instance: &Path,
        moves: &[(PathBuf, PathBuf)],
        placed: &mut Vec<PlacedItem>,
        created: &mut Vec<PathBuf>,
        progress: &mut impl FnMut(CoreProgress),
    ) -> Result<(), LauncherError> {
        for (from, to) in moves {
            if let Some(parent) = to.parent() {
                Self::make_dir(parent, created)?;
            }
            let replaced = if to.exists() {
                let aside = Self::aside(to);
                std::fs::rename(to, &aside)?;
                Some(aside)
            } else {
                None
            };
            let item = PlacedItem { target: to.clone(), replaced };
            if let Err(error) = std::fs::rename(from, to) {
                placed.push(item);
                return Err(error.into());
            }
            placed.push(item);
        }
        progress(CoreProgress::Finishing);
        Self::make_dir(&instance.join("amml").join("cache"), created)?;
        Self::make_dir(&instance.join("mods").join("amml"), created)?;
        Ok(())
    }

    // creates a folder and every missing parent, remembering the ones it made
    fn make_dir(dir: &Path, created: &mut Vec<PathBuf>) -> Result<(), LauncherError> {
        let mut missing = Vec::new();
        let mut cursor = Some(dir);
        while let Some(path) = cursor {
            if path.exists() {
                break;
            }
            missing.push(path.to_path_buf());
            cursor = path.parent();
        }
        std::fs::create_dir_all(dir)?;
        created.extend(missing.into_iter().rev());
        Ok(())
    }

    // removes what was placed, puts what it replaced back, and removes the folders it made when they are empty
    fn undo(placed: &[PlacedItem], created: &[PathBuf]) {
        for item in placed.iter().rev() {
            Self::delete(&item.target);
            if let Some(old) = &item.replaced
                && let Err(error) = std::fs::rename(old, &item.target)
            {
                curia::error!("a replaced file was not restored", {
                    file: item.target.display().to_string(),
                    error: error.to_string(),
                });
            }
        }
        for dir in created.iter().rev() {
            let _ = std::fs::remove_dir(dir);
        }
    }

    fn aside(path: &Path) -> PathBuf {
        let mut name = path.file_name().map(|name| name.to_os_string()).unwrap_or_default();
        name.push(format!(".old-{}", uuid::Uuid::new_v4().simple()));
        path.with_file_name(name)
    }

    fn delete(path: &Path) {
        if !path.exists() {
            return;
        }
        let outcome = if path.is_dir() {
            TreeOps::remove(path).map_err(|error| error.to_string())
        } else {
            std::fs::remove_file(path).map_err(|error| error.to_string())
        };
        if let Err(error) = outcome {
            curia::warn!("a file was not deleted", { file: path.display().to_string(), error: error });
        }
    }
}
