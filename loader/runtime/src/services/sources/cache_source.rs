use std::path::PathBuf;

use amml_data::{BuildId, EntryFingerprints, EntryId, RecipeSet};
use amml_derive::DERIVER_VERSION;

use crate::services::KnownResults;
use crate::structs::{SourceNote, SourceOutcome};

/// Reads the results of one exe build already on disk: the loader's cache file and the shipped data file.
///
/// A hit is a stored value whose fingerprint equals the current fingerprint of its entry. The answer is never
/// final: an id that is not a hit is asked of the derive source, unless a stored failure settles it.
#[derive(Clone, Debug)]
pub struct CacheSource {
    cache_dir: PathBuf,
    data_dir: PathBuf,
    build: BuildId,
    set: RecipeSet,
}

impl CacheSource {
    /// `build` comes from the exe file on disk, never from the mapped image. `cache_dir` holds the loader's cache
    /// file, `data_dir` the shipped data files; `set` gives the current fingerprint and kind of each entry.
    pub fn new(cache_dir: PathBuf, data_dir: PathBuf, build: BuildId, set: RecipeSet) -> Self {
        Self { cache_dir, data_dir, build, set }
    }

    pub fn resolve_all(&self, wanted: &[EntryId]) -> SourceOutcome {
        let mut outcome = SourceOutcome::new();
        let prints = match EntryFingerprints::of(&self.set, DERIVER_VERSION) {
            Ok(prints) => prints,
            Err(error) => {
                outcome.notes.push(SourceNote::RecipesUnreadable(error.to_string()));
                return outcome;
            }
        };
        let known = KnownResults::read(&self.cache_dir, &self.data_dir, self.build, &self.set, &prints);
        let all = known.merged();
        outcome.notes = known.notes;
        outcome.found = all.entries;
        outcome.found = outcome.found_for(wanted);
        outcome.settled = all.failed.into_keys().collect();
        outcome.settled = outcome.settled_for(wanted);
        outcome
    }
}
