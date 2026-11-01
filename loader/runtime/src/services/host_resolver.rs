use std::collections::BTreeSet;

use amml_data::{EntryId, EntryKind, RecipeSet};
use amml_derive::LayoutGuard;

use super::{CacheSource, DeriveSource, LeviLaminaSource, ResolverChain, ResolverSource, SourceMemo};
use crate::structs::{EntryTable, SourceNote};

/// Owns the source order of a host. The derive run happens at most once per `resolve`.
pub struct HostResolver {
    cache: Option<ResolverSource>,
    derive: Option<ResolverSource>,
    levilamina: Option<ResolverSource>,
    set: RecipeSet,
}

impl HostResolver {
    /// Functions: LeviLamina, checked against the cache or derive value when there is one; the two
    /// must agree, and an entry they disagree on is missing. Every other kind: cache, derive, LeviLamina.
    pub fn levilamina(cache: CacheSource, derive: DeriveSource, levilamina: LeviLaminaSource, set: RecipeSet) -> Self {
        Self {
            cache: Some(ResolverSource::Cache(cache)),
            derive: Some(ResolverSource::Derive(derive)),
            levilamina: Some(ResolverSource::LeviLamina(levilamina)),
            set,
        }
    }

    /// LeviLamina only, for a host that can read neither the exe build nor the recipes.
    pub fn levilamina_only(levilamina: LeviLaminaSource, set: RecipeSet) -> Self {
        Self { cache: None, derive: None, levilamina: Some(ResolverSource::LeviLamina(levilamina)), set }
    }

    /// Cache, derive, for every kind.
    pub fn standalone(cache: CacheSource, derive: DeriveSource, set: RecipeSet) -> Self {
        Self {
            cache: Some(ResolverSource::Cache(cache)),
            derive: Some(ResolverSource::Derive(derive)),
            levilamina: None,
            set,
        }
    }

    /// The table of `wanted`. A wanted entry that an `entry_is` check refuses on this build (its guard resolved to
    /// another value) is recorded as not for this build, not as missing. The guards are resolved for that judgement
    /// only and are not in the table unless they are wanted too.
    pub fn resolve(&self, wanted: &[EntryId]) -> EntryTable {
        let asked: Vec<EntryId> = wanted.iter().cloned().collect::<BTreeSet<_>>().into_iter().collect();
        let mut table = self.resolve_with_guards(&self.with_guards(&asked));
        for id in table.missing() {
            let Some(entry) = self.set.get(&id) else { continue };
            if let Some(refusal) = LayoutGuard::refusal(entry, |guard| table.get(guard).map(|resolved| resolved.value))
            {
                table.refuse(id, refusal);
            }
        }
        table.narrowed(&asked)
    }

    // `wanted` and every entry its `entry_is` checks read, followed through the guards of the guards, sorted
    fn with_guards(&self, wanted: &[EntryId]) -> Vec<EntryId> {
        let mut all: BTreeSet<EntryId> = wanted.iter().cloned().collect();
        let mut todo: Vec<EntryId> = wanted.to_vec();
        while let Some(id) = todo.pop() {
            let Some(entry) = self.set.get(&id) else { continue };
            for guard in LayoutGuard::guards(entry) {
                if all.insert(guard.clone()) {
                    todo.push(guard.clone());
                }
            }
        }
        all.into_iter().collect()
    }

    fn resolve_with_guards(&self, wanted: &[EntryId]) -> EntryTable {
        let wanted = wanted.to_vec();
        let mut table = EntryTable::new(&wanted);
        let mut memo = SourceMemo::default();
        match &self.levilamina {
            Some(levilamina) => {
                let (functions, others): (Vec<EntryId>, Vec<EntryId>) = wanted
                    .into_iter()
                    .partition(|id| self.set.get(id).is_some_and(|entry| entry.kind == EntryKind::Function));
                self.resolve_functions(levilamina, &functions, &mut table, &mut memo);
                let mut other_order = self.cache_and_derive();
                other_order.push(levilamina);
                ResolverChain::run(&other_order, &others, &mut table, &mut memo);
            }
            None => ResolverChain::run(&self.cache_and_derive(), &wanted, &mut table, &mut memo),
        }
        table
    }

    // LeviLamina's value for a function is never taken alone when the cache or derive has one: both
    // are asked, and a pair that differs leaves the entry missing.
    fn resolve_functions(
        &self,
        levilamina: &ResolverSource,
        functions: &[EntryId],
        table: &mut EntryTable,
        memo: &mut SourceMemo,
    ) {
        let mut theirs = EntryTable::new(functions);
        ResolverChain::run(&[levilamina], functions, &mut theirs, memo);
        let mut ours = EntryTable::new(functions);
        ResolverChain::run(&self.cache_and_derive(), functions, &mut ours, memo);
        table.add_notes(theirs.notes().to_vec());
        table.add_notes(ours.notes().to_vec());
        for id in functions {
            match (theirs.get(id), ours.get(id)) {
                (Some(levilamina), Some(derived)) if levilamina.value != derived.value => {
                    table.add_notes(vec![SourceNote::SourcesDisagree {
                        id: id.clone(),
                        levilamina: levilamina.value,
                        derived: derived.value,
                    }]);
                }
                (Some(levilamina), _) => table.insert(id.clone(), levilamina, "levilamina"),
                (None, Some(derived)) => {
                    table.insert(id.clone(), derived, ours.source_of(id).unwrap_or("cache"));
                }
                (None, None) => {}
            }
        }
    }

    fn cache_and_derive(&self) -> Vec<&ResolverSource> {
        self.cache.iter().chain(self.derive.iter()).collect()
    }
}
