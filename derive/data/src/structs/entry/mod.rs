pub mod id;
pub mod kind;

pub use id::EntryId;
pub use kind::EntryKind;

use super::{Check, Recipe};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: EntryId,
    pub kind: EntryKind,
    pub msvc_name: Option<String>,
    pub recipes: Vec<Recipe>,
    pub checks: Vec<Check>,
    pub notes: String,
    /// Some mod detours this function, so `amml-derive signature` watches its signature across builds.
    pub hooked: bool,
}

impl Entry {
    /// The entries named by `ref` and `slot_of` steps in any recipe, in recipe order, then the vtables named by checks.
    pub fn references(&self) -> impl Iterator<Item = &EntryId> {
        let steps =
            self.recipes.iter().flat_map(|recipe| recipe.steps.iter()).filter_map(|step| step.referenced_entry());
        steps.chain(self.checks.iter().filter_map(|check| check.referenced_entry()))
    }
}
