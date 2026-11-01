use amml_data::{EntryId, EntryKind, RecipeSet, Resolved};

use crate::structs::{ImageLayout, SourceNote, SourceOutcome};

/// Finds the absolute address of an MSVC-decorated symbol, as LeviLamina's `pl_resolve_symbol` does.
pub type SymbolLookup = fn(&str) -> Option<u64>;

/// Asks LeviLamina for functions and globals that name an `msvc_name`.
#[derive(Clone, Debug)]
pub struct LeviLaminaSource {
    lookup: SymbolLookup,
    set: RecipeSet,
    module_base: u64,
    layout: ImageLayout,
}

impl LeviLaminaSource {
    /// `layout` is the image at `module_base`; an address outside it is never used.
    pub fn new(lookup: SymbolLookup, set: RecipeSet, module_base: u64, layout: ImageLayout) -> Self {
        Self { lookup, set, module_base, layout }
    }

    /// Values are RVAs, and `recipes_agreeing` is 0 because they are not derived.
    /// An address is kept only when it lies inside the image and, for a `function`, inside an
    /// executable section; any other is dropped with a note.
    pub fn resolve_all(&self, wanted: &[EntryId]) -> SourceOutcome {
        let mut outcome = SourceOutcome::new();
        for id in wanted {
            let Some(entry) = self.set.get(id) else { continue };
            if !matches!(entry.kind, EntryKind::Function | EntryKind::Global) {
                continue;
            }
            let Some(name) = entry.msvc_name.as_deref() else { continue };
            let Some(address) = (self.lookup)(name) else {
                outcome.notes.push(SourceNote::LookupMissing(id.clone()));
                continue;
            };
            let Some(rva) = address.checked_sub(self.module_base).filter(|rva| self.layout.contains(*rva)) else {
                outcome.notes.push(SourceNote::LookupOutsideImage {
                    id: id.clone(),
                    address,
                    module_base: self.module_base,
                    size_of_image: self.layout.size_of_image(),
                });
                continue;
            };
            if entry.kind == EntryKind::Function && !self.layout.is_executable(rva) {
                outcome.notes.push(SourceNote::LookupNotExecutable { id: id.clone(), rva });
                continue;
            }
            outcome.found.insert(id.clone(), Resolved { kind: entry.kind, value: rva, recipes_agreeing: 0 });
        }
        outcome
    }
}
