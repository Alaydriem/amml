use std::collections::{BTreeMap, BTreeSet};

use amml_data::{EntryId, Resolved};

use super::SourceNote;

/// What one source found, what it wants the host to know, and which failures are already known.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceOutcome {
    pub found: BTreeMap<EntryId, Resolved>,
    pub notes: Vec<SourceNote>,
    /// Ids with a stored failure for this build and these recipes; a derive source after this one is not asked
    /// for them.
    pub settled: BTreeSet<EntryId>,
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

    /// The settled ids that are in `wanted`.
    pub fn settled_for(&self, wanted: &[EntryId]) -> BTreeSet<EntryId> {
        wanted.iter().filter(|id| self.settled.contains(id)).cloned().collect()
    }
}
