use crate::LauncherError;
use amml_gdk::TreeOps;
use std::path::{Path, PathBuf};

// Replaces a mod folder with a freshly unpacked one in steps that can each be undone: the mod's `config\` and `data\`
// move into the new folder, the old folder goes aside, the new folder takes its place.
pub struct ModSwap {
    staging: PathBuf,
    aside: PathBuf,
    existing: Option<PathBuf>,
    moved: Vec<&'static str>,
    aside_done: bool,
    placed: bool,
}

impl ModSwap {
    // the folders of a mod that an update keeps
    const KEPT: [&'static str; 2] = ["config", "data"];

    pub fn new(staging: PathBuf, aside: PathBuf, existing: Option<PathBuf>) -> ModSwap {
        ModSwap { staging, aside, existing, moved: Vec::new(), aside_done: false, placed: false }
    }

    // `target` is where the new folder ends up: the old folder's place, or the enabled place for a new mod
    pub fn run(&mut self, target: &Path) -> Result<(), LauncherError> {
        if let Some(existing) = self.existing.clone() {
            for part in Self::KEPT {
                let kept = existing.join(part);
                if kept.is_dir() {
                    let slot = self.staging.join(part);
                    if slot.exists() {
                        TreeOps::remove(&slot)?;
                    }
                    std::fs::rename(&kept, &slot)?;
                    self.moved.push(part);
                }
            }
            std::fs::rename(&existing, &self.aside)?;
            self.aside_done = true;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&self.staging, target)?;
        self.placed = true;
        Ok(())
    }

    // the change is final: the old folder is deleted
    pub fn finish(&mut self) {
        if self.aside_done
            && let Err(error) = TreeOps::remove(&self.aside)
        {
            curia::warn!("the old mod folder was not deleted", {
                folder: self.aside.display().to_string(),
                error: error.to_string(),
            });
        }
        self.aside_done = false;
    }

    // Puts everything back as it was, then removes the staging folder. When a restore step fails, staging may hold
    // the only copy of the mod's `config\` and `data\`, so it is kept and its path is logged.
    pub fn undo(&mut self, target: &Path) {
        let mut restored = true;
        if self.placed {
            restored &=
                Self::step(std::fs::rename(target, &self.staging), "the new mod folder was not taken back", target);
            self.placed = false;
        }
        if let Some(existing) = &self.existing {
            if self.aside_done {
                restored &=
                    Self::step(std::fs::rename(&self.aside, existing), "the old mod folder was not restored", existing);
                self.aside_done = false;
            }
            for part in self.moved.iter().rev() {
                let from = self.staging.join(part);
                let to = existing.join(part);
                restored &= Self::step(std::fs::rename(&from, &to), "a kept folder was not restored", &to);
            }
            self.moved.clear();
        }
        if !restored {
            curia::error!("the staging folder is kept because a restore step failed", {
                folder: self.staging.display().to_string(),
            });
            return;
        }
        if let Err(error) = TreeOps::remove(&self.staging) {
            curia::warn!("the staging folder was not deleted", {
                folder: self.staging.display().to_string(),
                error: error.to_string(),
            });
        }
    }

    // `true` when the step worked
    fn step(outcome: std::io::Result<()>, what: &str, path: &Path) -> bool {
        match outcome {
            Ok(()) => true,
            Err(error) => {
                curia::error!("an undo step failed", {
                    step: what.to_string(),
                    path: path.display().to_string(),
                    error: error.to_string(),
                });
                false
            }
        }
    }
}
