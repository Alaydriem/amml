use std::collections::BTreeMap;

use amml_data::{BuildData, Check, DataError, DependencyOrder, Entry, EntryId, EntryKind, RecipeSet, Resolved, Step};

use crate::errors::StepError;
use crate::services::{CheckRunner, RecipeRunner, RttiIndex, StringXref};
use crate::structs::{Degraded, DeriveOutcome, Failure, FailureReason, Image};

// the resolved value, the errors of the recipes that failed (by recipe index) and the number of counted checks
type Resolution = (Resolved, Vec<(usize, StepError)>, usize);

pub struct Deriver<'a> {
    image: &'a Image,
    set: &'a RecipeSet,
    rtti: RttiIndex,
    xrefs: StringXref,
}

impl<'a> Deriver<'a> {
    /// Agreeing recipes a function with only `section` checks, or a vslot with only `range` checks, needs.
    pub const WEAK_CHECKS_ONLY_RECIPES: u32 = 2;

    pub fn new(image: &'a Image, set: &'a RecipeSet) -> Self {
        Self { image, set, rtti: RttiIndex::new(image), xrefs: StringXref::new(image) }
    }

    /// A runner over this deriver's image and indexes, for tools that run single recipes.
    pub fn runner(&self) -> RecipeRunner<'_> {
        RecipeRunner::new(self.image, &self.rtti, &self.xrefs)
    }

    /// Fails only on a dependency cycle; every per-entry problem is a `Failure`.
    pub fn derive(&self) -> Result<DeriveOutcome, DataError> {
        let order = DependencyOrder::of(self.set)?;
        let runner = self.runner();
        let mut resolved: BTreeMap<EntryId, Resolved> = BTreeMap::new();
        let mut failures = Vec::new();
        let mut degraded = Vec::new();
        for id in order {
            let Some(entry) = self.set.get(&id) else { continue };
            match self.resolve(&runner, entry, &resolved) {
                Ok((value, recipe_errors, counted_checks)) => {
                    let unverified = value.recipes_agreeing == 1 && counted_checks == 0;
                    if !recipe_errors.is_empty() || unverified {
                        degraded.push(Degraded::new(id.clone(), recipe_errors, unverified));
                    }
                    resolved.insert(id, value);
                }
                Err(reason) => failures.push(Failure::new(id, reason)),
            }
        }
        let mut data = BuildData::new(self.image.build());
        data.entries = resolved;
        Ok(DeriveOutcome::new(data, failures, degraded))
    }

    // the resolved value, the errors of the recipes that failed (by recipe index) and the number of checks that
    // count as evidence (see `counted_checks`)
    fn resolve(
        &self,
        runner: &RecipeRunner,
        entry: &Entry,
        resolved: &BTreeMap<EntryId, Resolved>,
    ) -> Result<Resolution, FailureReason> {
        let mut values = Vec::new();
        let mut non_vtable_succeeded = false;
        let mut all_end_in_start_of = true;
        let mut errors: Vec<(usize, StepError)> = Vec::new();
        for (index, recipe) in entry.recipes.iter().enumerate() {
            match runner.run(recipe, entry.kind, resolved) {
                Ok(value) => {
                    non_vtable_succeeded |= !recipe.steps.iter().any(|step| matches!(step, Step::VtableSlot { .. }));
                    all_end_in_start_of &= matches!(recipe.steps.last(), Some(Step::FunctionStartOf));
                    values.push(value);
                }
                Err(error) => errors.push((index, error)),
            }
        }
        let agreeing = u32::try_from(values.len()).unwrap_or(u32::MAX);
        values.sort_unstable();
        values.dedup();
        let value = match values.as_slice() {
            [] => return Err(Self::all_failed(errors)),
            [only] => *only,
            _ => return Err(FailureReason::Disagree(values)),
        };
        // a vtable route alone cannot tell a shifted slot from the right one
        if !non_vtable_succeeded {
            return Err(FailureReason::VtableRouteOnly);
        }
        let counted = Self::counted_checks(entry, all_end_in_start_of);
        // a section or range check only bounds the value, so one recipe must not carry the entry alone
        if Self::weak_checks_only(entry, &counted) && agreeing < Self::WEAK_CHECKS_ONLY_RECIPES {
            return Err(FailureReason::WeakChecksOnly { agreeing });
        }
        // a check that reads another entry cannot judge the value without it
        for check in &entry.checks {
            if let Some(target) = check.referenced_entry()
                && !resolved.contains_key(target)
            {
                return Err(FailureReason::DependencyFailed(target.clone()));
            }
        }
        // every recipe that succeeded reached `value`, so the count before dedup is the agreement
        for check in &entry.checks {
            if !CheckRunner::run(self.image, &self.rtti, check, entry.kind, value, resolved) {
                return Err(FailureReason::CheckFailed(check.clone()));
            }
        }
        Ok((Resolved { kind: entry.kind, value, recipes_agreeing: agreeing }, errors, counted.len()))
    }

    // the checks that count as evidence: `function_start` is always true after `function_start_of`, so it does not
    // count when every recipe that succeeded ends with that step; `entry_is` only says which layout the build has,
    // so it never counts
    fn counted_checks(entry: &Entry, all_end_in_start_of: bool) -> Vec<&Check> {
        entry
            .checks
            .iter()
            .filter(|check| !(all_end_in_start_of && matches!(check, Check::FunctionStart)))
            .filter(|check| !matches!(check, Check::EntryIs { .. }))
            .collect()
    }

    // a function whose counted checks are all `section`, or a vslot whose checks are all `range`; globals and fields
    // are exempt
    fn weak_checks_only(entry: &Entry, counted: &[&Check]) -> bool {
        let weak = |check: &&Check| match entry.kind {
            EntryKind::Function => matches!(check, Check::Section { .. }),
            EntryKind::VSlot => matches!(check, Check::Range { .. }),
            _ => false,
        };
        !counted.is_empty() && counted.iter().all(weak)
    }

    // a dependency failure anywhere in the entry keeps the reason `DependencyFailed`, so the report marks it
    fn all_failed(errors: Vec<(usize, StepError)>) -> FailureReason {
        let dependency = errors.iter().find_map(|(_, error)| match error {
            StepError::DependencyFailed(target) => Some(target.clone()),
            _ => None,
        });
        match dependency {
            Some(target) => FailureReason::DependencyFailed(target),
            None => FailureReason::AllRecipesFailed(errors.into_iter().map(|(_, error)| error).collect()),
        }
    }
}
