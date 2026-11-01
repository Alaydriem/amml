use amml_data::{BuildId, BuildIdReader, RecipeLoader, RecipeSet};
use amml_runtime::RecipeDigest;

use crate::HostPaths;

/// What the host reads from disk before it resolves: the recipes, and the exe build and recipe
/// digest that name the cache file.
///
/// Anything unreadable degrades: the recipe set is empty, and the cache and derive sources are
/// left out, so only LeviLamina's lookup (when the host has it) resolves entries. Each degrade logs one line.
pub struct HostInputs {
    set: RecipeSet,
    cache_key: Option<(BuildId, String)>,
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
        let digest = match RecipeDigest::of_dir(&paths.recipes) {
            Ok(digest) => digest,
            Err(error) => {
                curia::warn!("recipes cannot be hashed; the cache and derive are not used", {
                    error: error.to_string()
                });
                return Self { set, cache_key: None };
            }
        };
        match BuildIdReader::from_path(&paths.exe) {
            Ok(build) => Self { set, cache_key: Some((build, digest)) },
            Err(error) => {
                curia::warn!("the game build cannot be read; the cache and derive are not used", {
                    exe: paths.exe.display().to_string(),
                    error: error.to_string()
                });
                Self { set, cache_key: None }
            }
        }
    }

    fn empty() -> Self {
        Self { set: RecipeSet::default(), cache_key: None }
    }

    pub fn set(&self) -> &RecipeSet {
        &self.set
    }

    /// The exe build and recipe digest, when the cache and derive sources can run.
    pub fn cache_key(&self) -> Option<&(BuildId, String)> {
        self.cache_key.as_ref()
    }
}
