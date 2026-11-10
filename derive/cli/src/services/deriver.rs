use std::collections::{BTreeMap, BTreeSet};

use amml_data::{
    BuildData, Check, DataError, DependencyOrder, Entry, EntryFingerprints, EntryId, EntryKind, RecipeSet, Resolved,
    Step,
};

use crate::DERIVER_VERSION;
use crate::errors::StepError;
use crate::services::{CheckRunner, RecipeRunner, ReportWriter, RttiIndex, StringXref};
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

    /// Derives every entry. Fails only on a dependency cycle; every per-entry problem is a `Failure`.
    pub fn derive(&self) -> Result<DeriveOutcome, DataError> {
        self.derive_with(&BuildData::new(self.image.build()), None)
    }

    /// Derives the entries of `solve` (every entry when `None`) and every entry they reference, transitively.
    ///
    /// An entry is reused from `known` only when `known` is for this image's build, its stored fingerprint equals
    /// the current one, and `known` holds exactly one record for it (`resolved` of the same kind, or `failed`).
    /// Every other entry is derived. Fails only on a dependency cycle.
    pub fn derive_with(
        &self,
        known: &BuildData,
        solve: Option<&BTreeSet<EntryId>>,
    ) -> Result<DeriveOutcome, DataError> {
        let order = DependencyOrder::of(self.set)?;
        let prints = EntryFingerprints::of(self.set, DERIVER_VERSION)?;
        let wanted = solve.map(|roots| self.closure(roots));
        let same_build = known.build == self.image.build();
        let runner = self.runner();
        let mut data = BuildData::new(self.image.build());
        let mut failures = Vec::new();
        let mut degraded = Vec::new();
        let mut reused = Vec::new();
        for id in order {
            if wanted.as_ref().is_some_and(|wanted| !wanted.contains(&id)) {
                continue;
            }
            let Some(entry) = self.set.get(&id) else { continue };
            let Some(print) = prints.get(&id).copied() else { continue };
            data.fingerprints.insert(id.clone(), print);
            if same_build && known.fingerprints.get(&id) == Some(&print) {
                match (known.entries.get(&id), known.failed.get(&id)) {
                    (Some(value), None) if value.kind == entry.kind => {
                        data.entries.insert(id.clone(), *value);
                        reused.push(id);
                        continue;
                    }
                    (None, Some(reason)) => {
                        data.failed.insert(id.clone(), reason.clone());
                        reused.push(id);
                        continue;
                    }
                    _ => {}
                }
            }
            match self.resolve(&runner, entry, &data.entries) {
                Ok((value, recipe_errors, counted_checks)) => {
                    let unverified = value.recipes_agreeing == 1 && counted_checks == 0;
                    if !recipe_errors.is_empty() || unverified {
                        degraded.push(Degraded::new(id.clone(), recipe_errors, unverified));
                    }
                    data.entries.insert(id, value);
                }
                Err(reason) => {
                    data.failed.insert(id.clone(), ReportWriter::reason(&reason));
                    failures.push(Failure::new(id, reason));
                }
            }
        }
        reused.sort();
        Ok(DeriveOutcome { data, failures, degraded, reused })
    }

    // `roots` and every entry they reference, transitively; ids not in the set are left out
    fn closure(&self, roots: &BTreeSet<EntryId>) -> BTreeSet<EntryId> {
        let mut found = BTreeSet::new();
        let mut pending: Vec<&EntryId> = roots.iter().collect();
        while let Some(id) = pending.pop() {
            let Some(entry) = self.set.get(id) else { continue };
            if found.insert(id.clone()) {
                pending.extend(entry.references());
            }
        }
        found
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
