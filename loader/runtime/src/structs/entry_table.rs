use std::collections::BTreeMap;

use amml_data::{EntryId, Resolved};
use amml_derive::LayoutRefusal;

use super::SourceNote;

/// The values found for the wanted entries and the name of the source that gave each one.
///
/// A wanted entry with no value is either not for this build (an `entry_is` check refused it: it belongs to the
/// other layout of the game) or missing. Both are absent for the mods; only a missing entry is a fault.
#[derive(Clone, Debug)]
pub struct EntryTable {
    wanted: Vec<EntryId>,
    found: BTreeMap<EntryId, (Resolved, &'static str)>,
    not_for_build: BTreeMap<EntryId, LayoutRefusal>,
    notes: Vec<SourceNote>,
}

impl EntryTable {
    /// An empty table for `wanted`; repeated ids count once and the order is sorted.
    pub fn new(wanted: &[EntryId]) -> Self {
        let mut wanted = wanted.to_vec();
        wanted.sort();
        wanted.dedup();
        Self { wanted, found: BTreeMap::new(), not_for_build: BTreeMap::new(), notes: Vec::new() }
    }

    /// Records `resolved` for `id` unless `id` is not wanted or already has a value.
    pub fn insert(&mut self, id: EntryId, resolved: Resolved, source: &'static str) {
        if self.wanted.binary_search(&id).is_ok() {
            self.found.entry(id).or_insert((resolved, source));
        }
    }

    pub fn get(&self, id: &EntryId) -> Option<Resolved> {
        self.found.get(id).map(|(resolved, _)| *resolved)
    }

    /// The wanted ids that have no value and are not refused as not for this build, sorted.
    pub fn missing(&self) -> Vec<EntryId> {
        self.wanted
            .iter()
            .filter(|id| !self.found.contains_key(*id) && !self.not_for_build.contains_key(*id))
            .cloned()
            .collect()
    }

    /// Records that `id` is not for this build, unless `id` is not wanted or has a value. The entry stays absent.
    pub fn refuse(&mut self, id: EntryId, refusal: LayoutRefusal) {
        if self.wanted.binary_search(&id).is_ok() && !self.found.contains_key(&id) {
            self.not_for_build.entry(id).or_insert(refusal);
        }
    }

    /// The wanted ids an `entry_is` check refused on this build, with the check, sorted by id.
    pub fn not_for_build(&self) -> &BTreeMap<EntryId, LayoutRefusal> {
        &self.not_for_build
    }

    /// The part of this table for `wanted` only: its values, refusals and every note.
    pub fn narrowed(&self, wanted: &[EntryId]) -> Self {
        let mut table = Self::new(wanted);
        for (id, (resolved, source)) in &self.found {
            table.insert(id.clone(), *resolved, source);
        }
        for (id, refusal) in &self.not_for_build {
            table.refuse(id.clone(), refusal.clone());
        }
        table.notes = self.notes.clone();
        table
    }

    /// `"cache"`, `"derive"` or `"levilamina"`.
    pub fn source_of(&self, id: &EntryId) -> Option<&'static str> {
        self.found.get(id).map(|(_, source)| *source)
    }

    pub fn add_notes(&mut self, notes: Vec<SourceNote>) {
        self.notes.extend(notes);
    }

    /// What the sources met, in the order they met it, for the host to log.
    pub fn notes(&self) -> &[SourceNote] {
        &self.notes
    }
}
