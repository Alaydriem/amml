use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{Check, Entry, EntryId, EntryKind, Recipe, Step, ValueType};
use crate::errors::DataError;

#[derive(Clone, Debug, Default)]
pub struct RecipeSet {
    entries: Vec<Entry>,
    sources: Vec<PathBuf>,
    root: Option<PathBuf>,
}

impl RecipeSet {
    /// Like `with_sources`, for entries that did not come from files.
    pub fn new(entries: Vec<Entry>) -> Result<RecipeSet, DataError> {
        Self::with_sources(entries, Vec::new())
    }

    /// Validates ids, recipes, first steps, references and value types, then keeps the entries in the given order.
    ///
    /// `sources[i]` is the file of `entries[i]` and is named in the errors of that entry.
    pub fn with_sources(entries: Vec<Entry>, sources: Vec<PathBuf>) -> Result<RecipeSet, DataError> {
        let path_of = |index: usize| sources.get(index).cloned();
        let mut kinds: BTreeMap<&EntryId, EntryKind> = BTreeMap::new();
        for (index, entry) in entries.iter().enumerate() {
            if !EntryId::is_valid_text(&entry.id.0) {
                return Err(DataError::Invalid {
                    entry: entry.id.clone(),
                    message: "ids may only use A-Z a-z 0-9 _ . : - and cannot be empty".to_string(),
                    path: path_of(index),
                });
            }
            if kinds.insert(&entry.id, entry.kind).is_some() {
                return Err(DataError::DuplicateId { id: entry.id.clone(), path: path_of(index) });
            }
        }
        for (index, entry) in entries.iter().enumerate() {
            if entry.recipes.is_empty() {
                return Err(DataError::EmptyRecipes { id: entry.id.clone(), path: path_of(index) });
            }
            for (recipe_index, recipe) in entry.recipes.iter().enumerate() {
                let first_is_source = recipe.steps.first().is_some_and(|step| step.is_source());
                let rest_may_follow = recipe.steps.iter().skip(1).all(|step| step.may_follow());
                if !first_is_source || !rest_may_follow {
                    return Err(DataError::BadFirstStep {
                        entry: entry.id.clone(),
                        recipe: recipe_index,
                        path: path_of(index),
                    });
                }
            }
            for target in entry.references() {
                if !kinds.contains_key(target) {
                    return Err(DataError::UnknownRef {
                        entry: entry.id.clone(),
                        target: target.clone(),
                        path: path_of(index),
                    });
                }
            }
            for check in &entry.checks {
                let (name, allowed) = Self::allowed_kinds(check);
                if !allowed.contains(&entry.kind) {
                    return Err(DataError::Invalid {
                        entry: entry.id.clone(),
                        message: format!(
                            "check `{name}` is only valid on {} entries, not {}",
                            Self::kind_list(allowed),
                            entry.kind.name()
                        ),
                        path: path_of(index),
                    });
                }
                let needed = if matches!(check, Check::EntryIs { .. }) { EntryKind::Field } else { EntryKind::Global };
                if let Some(target) = check.referenced_entry()
                    && let Some(kind) = kinds.get(target)
                    && *kind != needed
                {
                    return Err(DataError::Invalid {
                        entry: entry.id.clone(),
                        message: format!(
                            "check `{name}` needs a {} entry but `{target}` is a {}",
                            needed.name(),
                            kind.name()
                        ),
                        path: path_of(index),
                    });
                }
            }
            for (recipe_index, recipe) in entry.recipes.iter().enumerate() {
                Self::check_types(entry, recipe, &kinds).map_err(|message| DataError::Invalid {
                    entry: entry.id.clone(),
                    message: format!("recipe {recipe_index}: {message}"),
                    path: path_of(index),
                })?;
            }
        }
        Ok(RecipeSet { entries, sources, root: None })
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The file the entry was read from; `None` for an entry that did not come from a file.
    pub fn source_of(&self, id: &EntryId) -> Option<&Path> {
        let index = self.entries.iter().position(|entry| &entry.id == id)?;
        self.sources.get(index).map(PathBuf::as_path)
    }

    /// Records the directory the files were loaded from, for `relative_source_of`.
    pub fn with_root(mut self, root: PathBuf) -> RecipeSet {
        self.root = Some(root);
        self
    }

    /// The file of the entry relative to the loaded directory, with `/` separators. Falls back to the path as
    /// loaded when no directory is known.
    pub fn relative_source_of(&self, id: &EntryId) -> Option<String> {
        let source = self.source_of(id)?;
        let relative = self.root.as_deref().and_then(|root| source.strip_prefix(root).ok()).unwrap_or(source);
        Some(relative.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"))
    }

    pub fn get(&self, id: &EntryId) -> Option<&Entry> {
        self.entries.iter().find(|entry| &entry.id == id)
    }

    // the check name and the entry kinds it can judge
    fn allowed_kinds(check: &Check) -> (&'static str, &'static [EntryKind]) {
        match check {
            Check::FunctionStart => ("function_start", &[EntryKind::Function]),
            Check::LeafStart => ("leaf_start", &[EntryKind::Function]),
            Check::InVtableOf { .. } => ("in_vtable_of", &[EntryKind::Function]),
            Check::InVtable { .. } => ("in_vtable", &[EntryKind::Function]),
            Check::Section { .. } => ("section", &[EntryKind::Function, EntryKind::Global]),
            Check::Range { .. } => ("range", &[EntryKind::VSlot, EntryKind::Field, EntryKind::Global]),
            Check::EntryIs { .. } => {
                ("entry_is", &[EntryKind::Function, EntryKind::VSlot, EntryKind::Global, EntryKind::Field])
            }
        }
    }

    fn kind_list(kinds: &[EntryKind]) -> String {
        let names: Vec<_> = kinds.iter().map(|kind| kind.name()).collect();
        match names.split_last() {
            Some((last, [])) => (*last).to_string(),
            Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
            None => String::new(),
        }
    }

    // follows the value type through the steps and requires it to fit the entry kind
    fn check_types(entry: &Entry, recipe: &Recipe, kinds: &BTreeMap<&EntryId, EntryKind>) -> Result<(), String> {
        let mut current = None;
        for step in &recipe.steps {
            if let Step::SlotOf { vtable } = step
                && let Some(kind) = kinds.get(vtable)
                && *kind != EntryKind::Global
            {
                return Err(format!("step `slot_of` needs a global entry but `{vtable}` is a {}", kind.name()));
            }
            let referenced =
                step.referenced_entry().and_then(|target| kinds.get(target)).map(|kind| ValueType::of_kind(*kind));
            current = Some(step.result_type(current, referenced)?);
        }
        let expected = ValueType::of_kind(entry.kind);
        match current {
            Some(last) if last != expected => Err(format!(
                "a {} entry needs a recipe that ends in {} but this one ends in {}",
                entry.kind.name(),
                expected.label(),
                last.label()
            )),
            _ => Ok(()),
        }
    }
}
