use std::collections::{BTreeMap, BTreeSet};

use crate::errors::DataError;
use crate::structs::{EntryId, RecipeSet};

pub struct DependencyOrder;

impl DependencyOrder {
    // Orders entries so every `ref` target comes before its dependents; ties sort by id.
    pub fn of(set: &RecipeSet) -> Result<Vec<EntryId>, DataError> {
        let mut waiting_on: BTreeMap<EntryId, BTreeSet<EntryId>> = BTreeMap::new();
        let mut dependents: BTreeMap<EntryId, Vec<EntryId>> = BTreeMap::new();
        for entry in set.entries() {
            let targets: BTreeSet<EntryId> = entry.references().cloned().collect();
            for target in &targets {
                dependents.entry(target.clone()).or_default().push(entry.id.clone());
            }
            waiting_on.insert(entry.id.clone(), targets);
        }

        let mut ready: BTreeSet<EntryId> =
            waiting_on.iter().filter(|(_, targets)| targets.is_empty()).map(|(id, _)| id.clone()).collect();
        let mut order = Vec::with_capacity(waiting_on.len());
        while let Some(next) = ready.pop_first() {
            for dependent in dependents.get(&next).into_iter().flatten() {
                if let Some(targets) = waiting_on.get_mut(dependent) {
                    targets.remove(&next);
                    if targets.is_empty() {
                        ready.insert(dependent.clone());
                    }
                }
            }
            waiting_on.remove(&next);
            order.push(next);
        }

        if waiting_on.is_empty() { Ok(order) } else { Err(DataError::Cycle(waiting_on.into_keys().collect())) }
    }
}
