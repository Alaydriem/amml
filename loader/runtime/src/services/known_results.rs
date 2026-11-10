use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use amml_data::{BuildData, BuildDataStore, BuildId, DataError, EntryId, Fingerprint, RecipeSet};

use super::CacheFile;
use crate::structs::SourceNote;

/// The results of one build already on disk: the loader's cache file and the shipped data file, each cut down to
/// the records that are still valid. A record is valid when its id is in the recipe set, its stored fingerprint
/// equals the current one, the file holds exactly one record for it, and a resolved value has the entry's kind.
pub(crate) struct KnownResults {
    pub(crate) cache: BuildData,
    pub(crate) shipped: BuildData,
    pub(crate) notes: Vec<SourceNote>,
}

impl KnownResults {
    /// Reads `<cache_dir>/<stem>.hcl` and `<data_dir>/<stem>.hcl`. A missing, unreadable or foreign file gives no
    /// records; only a missing shipped file goes without a note.
    pub(crate) fn read(
        cache_dir: &Path,
        data_dir: &Path,
        build: BuildId,
        set: &RecipeSet,
        prints: &BTreeMap<EntryId, Fingerprint>,
    ) -> Self {
        let mut notes = Vec::new();
        let cache = match BuildDataStore::read(&CacheFile::path(cache_dir, build)) {
            Ok(data) if data.build == build => Self::valid(data, set, prints, &mut notes),
            Ok(data) => {
                notes
                    .push(SourceNote::CacheForeignBuild { found: data.build.file_stem(), expected: build.file_stem() });
                BuildData::new(build)
            }
            Err(error) => {
                notes.push(if Self::not_found(&error) {
                    SourceNote::CacheMissing
                } else {
                    SourceNote::CacheUnreadable(error.to_string())
                });
                BuildData::new(build)
            }
        };
        let shipped = match BuildDataStore::read(&CacheFile::path(data_dir, build)) {
            Ok(data) if data.build == build => Self::valid(data, set, prints, &mut notes),
            Ok(data) => {
                notes.push(SourceNote::ShippedForeignBuild {
                    found: data.build.file_stem(),
                    expected: build.file_stem(),
                });
                BuildData::new(build)
            }
            Err(error) => {
                if !Self::not_found(&error) {
                    notes.push(SourceNote::ShippedUnreadable(error.to_string()));
                }
                BuildData::new(build)
            }
        };
        Self { cache, shipped, notes }
    }

    /// Both files as one: the cache record of an id when there is one, else the shipped record.
    pub(crate) fn merged(&self) -> BuildData {
        let mut all = self.cache.clone();
        let records = self.shipped.entries.keys().chain(self.shipped.failed.keys());
        for id in records {
            if all.entries.contains_key(id) || all.failed.contains_key(id) {
                continue;
            }
            if let Some(resolved) = self.shipped.entries.get(id) {
                all.entries.insert(id.clone(), *resolved);
            }
            if let Some(reason) = self.shipped.failed.get(id) {
                all.failed.insert(id.clone(), reason.clone());
            }
            if let Some(print) = self.shipped.fingerprints.get(id) {
                all.fingerprints.insert(id.clone(), *print);
            }
        }
        all
    }

    fn not_found(error: &DataError) -> bool {
        matches!(error, DataError::Io { source, .. } if source.kind() == ErrorKind::NotFound)
    }

    fn valid(
        data: BuildData,
        set: &RecipeSet,
        prints: &BTreeMap<EntryId, Fingerprint>,
        notes: &mut Vec<SourceNote>,
    ) -> BuildData {
        let mut kept = BuildData::new(data.build);
        // the current fingerprint of `id` when the file stores the same one
        let current = |id: &EntryId| -> Option<Fingerprint> {
            let print = prints.get(id)?;
            (set.get(id).is_some() && data.fingerprints.get(id) == Some(print)).then_some(*print)
        };
        for (id, resolved) in &data.entries {
            let Some(print) = current(id) else { continue };
            if data.failed.contains_key(id) {
                continue;
            }
            if set.get(id).is_some_and(|entry| entry.kind != resolved.kind) {
                notes.push(SourceNote::CacheKindMismatch(id.clone()));
                continue;
            }
            kept.entries.insert(id.clone(), *resolved);
            kept.fingerprints.insert(id.clone(), print);
        }
        for (id, reason) in &data.failed {
            let Some(print) = current(id) else { continue };
            if data.entries.contains_key(id) {
                continue;
            }
            kept.failed.insert(id.clone(), reason.clone());
            kept.fingerprints.insert(id.clone(), print);
        }
        kept
    }
}
