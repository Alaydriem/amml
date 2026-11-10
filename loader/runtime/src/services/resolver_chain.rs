use std::collections::BTreeSet;

use amml_data::EntryId;

use super::{ResolverSource, SourceMemo};
use crate::structs::EntryTable;

/// Asks each source in order for the entries the sources before it did not find.
pub struct ResolverChain {
    sources: Vec<ResolverSource>,
}

impl ResolverChain {
    pub fn new(sources: Vec<ResolverSource>) -> Self {
        Self { sources }
    }

    pub fn resolve(&self, wanted: &[EntryId]) -> EntryTable {
        let mut table = EntryTable::new(wanted);
        let mut memo = SourceMemo::new(wanted);
        let order: Vec<&ResolverSource> = self.sources.iter().collect();
        Self::run(&order, wanted, &mut table, &mut memo);
        table
    }

    // An id whose failure an earlier source settled (a stored failure) is not asked of a derive source; the other
    // sources still get it. A source with nothing left to ask is skipped, so derive runs only for misses.
    pub(crate) fn run(order: &[&ResolverSource], ids: &[EntryId], table: &mut EntryTable, memo: &mut SourceMemo) {
        let mut settled: BTreeSet<EntryId> = BTreeSet::new();
        for source in order {
            let derive = matches!(source, ResolverSource::Derive(_));
            let still_missing: Vec<EntryId> = ids
                .iter()
                .filter(|id| table.get(id).is_none() && !(derive && settled.contains(*id)))
                .cloned()
                .collect();
            if still_missing.is_empty() {
                continue;
            }
            let (found, known_failed, notes) = memo.ask(source, &still_missing);
            table.add_notes(notes);
            for (id, resolved) in found {
                table.insert(id, resolved, source.name());
            }
            settled.extend(known_failed);
        }
    }
}
