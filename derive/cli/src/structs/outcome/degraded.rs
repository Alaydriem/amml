use amml_data::EntryId;

use crate::errors::StepError;

/// An entry that resolved but needs attention: a recipe of it failed, or nothing but one recipe vouches for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Degraded {
    pub id: EntryId,
    // index of each failed recipe in the entry, with its error
    pub recipe_errors: Vec<(usize, StepError)>,
    // resolved by a single recipe and no checks
    pub unverified: bool,
}

impl Degraded {
    pub fn new(id: EntryId, recipe_errors: Vec<(usize, StepError)>, unverified: bool) -> Self {
        Self { id, recipe_errors, unverified }
    }
}
