use std::collections::{BTreeMap, BTreeSet};

use amml_data::{EntryId, Resolved};

use super::SourceNote;

/// What one source found, what it wants the host to know, and whether its answer is final.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceOutcome {
    pub found: BTreeMap<EntryId, Resolved>,
    pub notes: Vec<SourceNote>,
    /// True when ids missing from `found` must not be derived again. Only a valid cache file sets it.
    pub authoritative: bool,
}

impl SourceOutcome {
    pub fn new() -> Self {
        Self::default()
    }

    /// The found values of the ids in `wanted`.
    pub fn found_for(&self, wanted: &[EntryId]) -> BTreeMap<EntryId, Resolved> {
        let wanted: BTreeSet<&EntryId> = wanted.iter().collect();
        self.found.iter().filter(|(id, _)| wanted.contains(id)).map(|(id, resolved)| (id.clone(), *resolved)).collect()
    }
}
