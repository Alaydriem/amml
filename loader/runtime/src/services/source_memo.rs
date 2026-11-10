use std::collections::{BTreeMap, BTreeSet};

use amml_data::{EntryId, Resolved};

use super::ResolverSource;
use crate::structs::{SourceNote, SourceOutcome};

/// Keeps the outcome of the cache and derive sources for one resolve, so each runs at most once.
///
/// The first ask of either source covers every id of the resolve, not only the ids of that ask: a later ask for
/// other ids (the LeviLamina host asks for functions and other kinds apart) is then served from the same run.
pub(crate) struct SourceMemo {
    all: Vec<EntryId>,
    outcomes: BTreeMap<&'static str, SourceOutcome>,
}

impl SourceMemo {
    /// `all` is every id the resolve may ask for.
    pub(crate) fn new(all: &[EntryId]) -> Self {
        Self { all: all.to_vec(), outcomes: BTreeMap::new() }
    }

    /// The values of `wanted`, the ids of `wanted` whose failure is settled, and the notes not yet reported.
    pub(crate) fn ask(
        &mut self,
        source: &ResolverSource,
        wanted: &[EntryId],
    ) -> (BTreeMap<EntryId, Resolved>, BTreeSet<EntryId>, Vec<SourceNote>) {
        if let ResolverSource::LeviLamina(_) = source {
            let outcome = source.resolve_all(wanted);
            return (outcome.found, outcome.settled, outcome.notes);
        }
        let name = source.name();
        let fresh = !self.outcomes.contains_key(name);
        if fresh {
            self.outcomes.insert(name, source.resolve_all(&self.all));
        }
        let Some(outcome) = self.outcomes.get(name) else {
            return (BTreeMap::new(), BTreeSet::new(), Vec::new());
        };
        let notes = if fresh { outcome.notes.clone() } else { Vec::new() };
        (outcome.found_for(wanted), outcome.settled_for(wanted), notes)
    }
}
