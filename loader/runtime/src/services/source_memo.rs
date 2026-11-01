use std::collections::BTreeMap;

use amml_data::{EntryId, Resolved};

use super::ResolverSource;
use crate::structs::{SourceNote, SourceOutcome};

/// Keeps the full outcome of the cache and derive sources for one resolve, so each runs at most once.
#[derive(Default)]
pub(crate) struct SourceMemo {
    outcomes: BTreeMap<&'static str, SourceOutcome>,
}

impl SourceMemo {
    /// The values of `wanted`, whether the source is final, and the notes not yet reported.
    pub(crate) fn ask(
        &mut self,
        source: &ResolverSource,
        wanted: &[EntryId],
    ) -> (BTreeMap<EntryId, Resolved>, bool, Vec<SourceNote>) {
        let name = source.name();
        let fresh = !self.outcomes.contains_key(name);
        if fresh {
            let full = match source {
                ResolverSource::LeviLamina(_) => {
                    let outcome = source.resolve_all(wanted);
                    return (outcome.found, outcome.authoritative, outcome.notes);
                }
                ResolverSource::Cache(cache) => cache.run(),
                ResolverSource::Derive(derive) => derive.run(),
            };
            self.outcomes.insert(name, full);
        }
        let Some(outcome) = self.outcomes.get(name) else {
            return (BTreeMap::new(), false, Vec::new());
        };
        let notes = if fresh { outcome.notes.clone() } else { Vec::new() };
        (outcome.found_for(wanted), outcome.authoritative, notes)
    }
}
