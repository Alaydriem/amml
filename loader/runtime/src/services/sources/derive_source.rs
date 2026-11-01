use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_data::{BuildDataStore, EntryId, RecipeLoader};
use amml_derive::{Deriver, ImageLoader};

use crate::services::{CacheFile, RecipeDigest};
use crate::structs::{SourceNote, SourceOutcome};

/// Runs the deriver on the exe file and saves the result as the cache file for that build and recipe set.
#[derive(Clone, Debug)]
pub struct DeriveSource {
    exe: PathBuf,
    recipes: PathBuf,
    cache_out: PathBuf,
    runs: Arc<AtomicUsize>,
}

impl DeriveSource {
    /// `recipes` is a recipe directory; `cache_out` is the data directory that receives the cache file.
    pub fn new(exe: PathBuf, recipes: PathBuf, cache_out: PathBuf) -> Self {
        Self { exe, recipes, cache_out, runs: Arc::new(AtomicUsize::new(0)) }
    }

    /// How many times the deriver ran; shared by clones of this source.
    pub fn run_counter(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.runs)
    }

    pub fn resolve_all(&self, wanted: &[EntryId]) -> SourceOutcome {
        let mut outcome = self.run();
        outcome.found = outcome.found_for(wanted);
        outcome
    }

    /// Derives every entry. The cache file holds what resolved and passed its checks.
    pub fn run(&self) -> SourceOutcome {
        let mut outcome = SourceOutcome::new();
        self.runs.fetch_add(1, Ordering::Relaxed);
        let digest = match RecipeDigest::of_dir(&self.recipes) {
            Ok(digest) => digest,
            Err(error) => {
                outcome.notes.push(SourceNote::RecipesUnreadable(error.to_string()));
                return outcome;
            }
        };
        let set = match RecipeLoader::load_dir(&self.recipes) {
            Ok(set) => set,
            Err(error) => {
                outcome.notes.push(SourceNote::RecipesUnreadable(error.to_string()));
                return outcome;
            }
        };
        let image = match ImageLoader::from_path(&self.exe) {
            Ok(image) => image,
            Err(error) => {
                outcome.notes.push(SourceNote::ImageUnreadable(error.to_string()));
                return outcome;
            }
        };
        let derived = match Deriver::new(&image, &set).derive() {
            Ok(derived) => derived,
            Err(error) => {
                outcome.notes.push(SourceNote::DeriveFailed(error.to_string()));
                return outcome;
            }
        };
        let path = CacheFile::path(&self.cache_out, derived.data.build, &digest);
        let written = std::fs::create_dir_all(&self.cache_out)
            .map_err(|error| error.to_string())
            .and_then(|()| BuildDataStore::write(&path, &derived.data).map_err(|error| error.to_string()));
        if let Err(message) = written {
            outcome.notes.push(SourceNote::CacheWriteFailed(message));
        }
        outcome.found = derived.data.entries;
        outcome
    }
}
