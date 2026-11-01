use std::io::ErrorKind;
use std::path::PathBuf;

use amml_data::{BuildDataStore, BuildId, DataError, EntryId, RecipeSet};

use crate::services::CacheFile;
use crate::structs::{SourceNote, SourceOutcome};

/// Reads the cache file of one exe build and one recipe set (see `CacheFile`).
///
/// A readable file for this build is final: ids it does not hold are missing, and derive is not asked.
#[derive(Clone, Debug)]
pub struct CacheSource {
    data_dir: PathBuf,
    build: BuildId,
    digest: String,
    set: RecipeSet,
}

impl CacheSource {
    /// `build` comes from the exe file on disk, never from the mapped image.
    /// `digest` is `RecipeDigest::of_dir` of the recipes; `set` gives the kind each cached entry must have.
    pub fn new(data_dir: PathBuf, build: BuildId, digest: String, set: RecipeSet) -> Self {
        Self { data_dir, build, digest, set }
    }

    pub fn resolve_all(&self, wanted: &[EntryId]) -> SourceOutcome {
        let mut outcome = self.run();
        outcome.found = outcome.found_for(wanted);
        outcome
    }

    /// Every valid cached entry, not only the wanted ones.
    pub fn run(&self) -> SourceOutcome {
        let mut outcome = SourceOutcome::new();
        let path = CacheFile::path(&self.data_dir, self.build, &self.digest);
        let data = match BuildDataStore::read(&path) {
            Ok(data) => data,
            Err(DataError::Io { source, .. }) if source.kind() == ErrorKind::NotFound => {
                outcome.notes.push(SourceNote::CacheMissing);
                return outcome;
            }
            Err(error) => {
                outcome.notes.push(SourceNote::CacheUnreadable(error.to_string()));
                return outcome;
            }
        };
        if data.build != self.build {
            outcome.notes.push(SourceNote::CacheForeignBuild {
                found: data.build.file_stem(),
                expected: self.build.file_stem(),
            });
            return outcome;
        }
        outcome.authoritative = true;
        for (id, resolved) in data.entries {
            match self.set.get(&id) {
                None => {}
                Some(entry) if entry.kind == resolved.kind => {
                    outcome.found.insert(id, resolved);
                }
                Some(_) => outcome.notes.push(SourceNote::CacheKindMismatch(id)),
            }
        }
        outcome
    }
}
