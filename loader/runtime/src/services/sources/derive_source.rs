use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use amml_data::{BuildData, BuildDataStore, EntryFingerprints, EntryId, RecipeLoader, RecipeSet};
use amml_derive::{DERIVER_VERSION, Deriver, Image, ImageLoader};

use crate::services::{CacheFile, KnownResults};
use crate::structs::{SourceNote, SourceOutcome};

/// Runs the deriver on the exe file, reusing the valid results already on disk, and keeps the cache file of that
/// build up to date.
#[derive(Clone, Debug)]
pub struct DeriveSource {
    exe: PathBuf,
    recipes: PathBuf,
    cache_dir: PathBuf,
    data_dir: PathBuf,
    runs: Arc<AtomicUsize>,
}

impl DeriveSource {
    /// `recipes` is a recipe directory; `cache_dir` receives the cache file; `data_dir` holds the shipped data files.
    pub fn new(exe: PathBuf, recipes: PathBuf, cache_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self { exe, recipes, cache_dir, data_dir, runs: Arc::new(AtomicUsize::new(0)) }
    }

    /// How many times the deriver ran; shared by clones of this source.
    pub fn run_counter(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.runs)
    }

    /// Derives `wanted` and what they reference, reusing every valid result of the cache and shipped files.
    ///
    /// The cache file is then rewritten as: every old cache record that is still valid, and every result derived in
    /// this run. Results reused from the shipped file are not copied into it.
    pub fn resolve_all(&self, wanted: &[EntryId]) -> SourceOutcome {
        let started = Instant::now();
        let mut outcome = SourceOutcome::new();
        let Some((set, image)) = self.inputs(&mut outcome) else {
            return outcome;
        };
        let prints = match EntryFingerprints::of(&set, DERIVER_VERSION) {
            Ok(prints) => prints,
            Err(error) => {
                outcome.notes.push(SourceNote::DeriveFailed(error.to_string()));
                return outcome;
            }
        };
        let build = image.build();
        let known = KnownResults::read(&self.cache_dir, &self.data_dir, build, &set, &prints);
        let solve: BTreeSet<EntryId> = wanted.iter().cloned().collect();
        let derived = match Deriver::new(&image, &set).derive_with(&known.merged(), Some(&solve)) {
            Ok(derived) => derived,
            Err(error) => {
                outcome.notes.push(SourceNote::DeriveFailed(error.to_string()));
                return outcome;
            }
        };
        let reused: BTreeSet<&EntryId> = derived.reused.iter().collect();
        let cache = Self::merge(known.cache, &derived.data, &reused);
        let path = CacheFile::path(&self.cache_dir, build);
        let written = std::fs::create_dir_all(&self.cache_dir)
            .map_err(|error| error.to_string())
            .and_then(|()| BuildDataStore::write(&path, &cache).map_err(|error| error.to_string()));
        if let Err(message) = written {
            outcome.notes.push(SourceNote::CacheWriteFailed(message));
        }
        let solved = derived.data.entries.len() + derived.data.failed.len();
        outcome.notes.push(SourceNote::DeriveRan {
            hits: reused.len(),
            derived: solved.saturating_sub(reused.len()),
            elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        });
        outcome.settled = derived.data.failed.keys().cloned().collect();
        outcome.settled = outcome.settled_for(wanted);
        outcome.found = derived.data.entries;
        outcome.found = outcome.found_for(wanted);
        outcome
    }

    /// Derives every entry with nothing reused, and writes no file. For measurements such as grade.
    pub fn run_full(&self) -> SourceOutcome {
        let mut outcome = SourceOutcome::new();
        let Some((set, image)) = self.inputs(&mut outcome) else {
            return outcome;
        };
        match Deriver::new(&image, &set).derive() {
            Ok(derived) => {
                outcome.settled = derived.data.failed.into_keys().collect();
                outcome.found = derived.data.entries;
            }
            Err(error) => outcome.notes.push(SourceNote::DeriveFailed(error.to_string())),
        }
        outcome
    }

    // counts the run, then loads the recipes and the exe image; a failure is noted in `outcome`
    fn inputs(&self, outcome: &mut SourceOutcome) -> Option<(RecipeSet, Image)> {
        self.runs.fetch_add(1, Ordering::Relaxed);
        let set = match RecipeLoader::load_dir(&self.recipes) {
            Ok(set) => set,
            Err(error) => {
                outcome.notes.push(SourceNote::RecipesUnreadable(error.to_string()));
                return None;
            }
        };
        match ImageLoader::from_path(&self.exe) {
            Ok(image) => Some((set, image)),
            Err(error) => {
                outcome.notes.push(SourceNote::ImageUnreadable(error.to_string()));
                None
            }
        }
    }

    // the still valid old cache records plus every record of `solved` that was derived, not reused
    fn merge(mut cache: BuildData, solved: &BuildData, reused: &BTreeSet<&EntryId>) -> BuildData {
        for (id, resolved) in &solved.entries {
            if reused.contains(id) {
                continue;
            }
            cache.failed.remove(id);
            cache.entries.insert(id.clone(), *resolved);
            if let Some(print) = solved.fingerprints.get(id) {
                cache.fingerprints.insert(id.clone(), *print);
            }
        }
        for (id, reason) in &solved.failed {
            if reused.contains(id) {
                continue;
            }
            cache.entries.remove(id);
            cache.failed.insert(id.clone(), reason.clone());
            if let Some(print) = solved.fingerprints.get(id) {
                cache.fingerprints.insert(id.clone(), *print);
            }
        }
        cache
    }
}
