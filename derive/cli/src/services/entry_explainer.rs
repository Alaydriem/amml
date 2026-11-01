use amml_data::{DataError, EntryId, RecipeSet};

use crate::services::Deriver;
use crate::structs::{DeriveOutcome, Image, RecipeTrace};

pub struct EntryExplainer<'a> {
    image: &'a Image,
    set: &'a RecipeSet,
}

impl<'a> EntryExplainer<'a> {
    pub fn new(image: &'a Image, set: &'a RecipeSet) -> Self {
        Self { image, set }
    }

    /// Runs every recipe of the entry step by step. The entries it refers to are derived first, so `ref` steps
    /// have their values. Fails when the entry is unknown or the set has a dependency cycle.
    pub fn explain(&self, id: &EntryId) -> Result<Vec<RecipeTrace>, DataError> {
        let deriver = Deriver::new(self.image, self.set);
        let outcome = deriver.derive()?;
        self.trace(id, &deriver, &outcome)
    }

    /// Like `explain`, for a caller that already has the outcome of a derive and the deriver that made it, so
    /// many entries can be traced without deriving the set again. Fails when the entry is unknown.
    pub fn trace(
        &self,
        id: &EntryId,
        deriver: &Deriver<'_>,
        outcome: &DeriveOutcome,
    ) -> Result<Vec<RecipeTrace>, DataError> {
        let entry = self.set.get(id).ok_or_else(|| DataError::UnknownEntry(id.clone()))?;
        let runner = deriver.runner();
        Ok(entry
            .recipes
            .iter()
            .enumerate()
            .map(|(index, recipe)| {
                let mut steps = Vec::new();
                let result = runner.run_explained(recipe, entry.kind, &outcome.data.entries, &mut steps);
                RecipeTrace { index, steps, result }
            })
            .collect())
    }
}
