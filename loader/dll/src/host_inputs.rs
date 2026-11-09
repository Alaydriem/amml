use amml_data::{BuildId, BuildIdReader, RecipeLoader, RecipeSet};

use crate::HostPaths;

/// What the host reads from disk before it resolves: the recipes, and the exe build that names the
/// cache and shipped data files.
///
/// Anything unreadable degrades: the recipe set is empty, and the cache and derive sources are
/// left out, so only LeviLamina's lookup (when the host has it) resolves entries. Each degrade logs one line.
pub struct HostInputs {
    set: RecipeSet,
    build: Option<BuildId>,
}

impl HostInputs {
    pub fn read(paths: &HostPaths) -> Self {
        let set = match RecipeLoader::load_dir(&paths.recipes) {
            Ok(set) => set,
            Err(error) => {
                curia::warn!("recipes unreadable; the cache and derive are not used", {
                    folder: paths.recipes.display().to_string(),
                    error: error.to_string()
                });
                return Self::empty();
            }
        };
        match BuildIdReader::from_path(&paths.exe) {
            Ok(build) => Self { set, build: Some(build) },
            Err(error) => {
                curia::warn!("the game build cannot be read; the cache and derive are not used", {
                    exe: paths.exe.display().to_string(),
                    error: error.to_string()
                });
                Self { set, build: None }
            }
        }
    }

    fn empty() -> Self {
        Self { set: RecipeSet::default(), build: None }
    }

    pub fn set(&self) -> &RecipeSet {
        &self.set
    }

    /// The exe build, when the cache and derive sources can run.
    pub fn build(&self) -> Option<BuildId> {
        self.build
    }
}
