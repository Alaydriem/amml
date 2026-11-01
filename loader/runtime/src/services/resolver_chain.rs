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
        let mut memo = SourceMemo::default();
        let order: Vec<&ResolverSource> = self.sources.iter().collect();
        Self::run(&order, wanted, &mut table, &mut memo);
        table
    }

    // Once a source is final (a valid cache file), derive sources after it are skipped.
    pub(crate) fn run(order: &[&ResolverSource], ids: &[EntryId], table: &mut EntryTable, memo: &mut SourceMemo) {
        let mut derive_settled = false;
        for source in order {
            if derive_settled && matches!(source, ResolverSource::Derive(_)) {
                continue;
            }
            let still_missing: Vec<EntryId> = ids.iter().filter(|id| table.get(id).is_none()).cloned().collect();
            if still_missing.is_empty() {
                break;
            }
            let (found, authoritative, notes) = memo.ask(source, &still_missing);
            table.add_notes(notes);
            for (id, resolved) in found {
                table.insert(id, resolved, source.name());
            }
            derive_settled |= authoritative;
        }
    }
}
