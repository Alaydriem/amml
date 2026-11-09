use std::path::{Path, PathBuf};

use amml_data::{EntryId, EntryKind, RecipeSet};

use super::{DeriveSource, LeviLaminaSource};
use crate::structs::{GradeReport, SourceNote};

/// Grades AMML's own addresses against LeviLamina's lookup.
///
/// The derived side is always a fresh derive of every entry: it reads no cache or shipped results and writes no
/// cache file. It runs apart from the resolver chain, so an entry that LeviLamina resolved for the mods is still
/// derived and compared.
pub struct Grade {
    derive: DeriveSource,
    levilamina: LeviLaminaSource,
    set: RecipeSet,
}

impl Grade {
    pub fn new(derive: DeriveSource, levilamina: LeviLaminaSource, set: RecipeSet) -> Self {
        Self { derive, levilamina, set }
    }

    /// The report, and the notes the sources met on the way.
    pub fn run(&self) -> (GradeReport, Vec<SourceNote>) {
        let mut derived = self.derive.run_full();
        let mut notes = std::mem::take(&mut derived.notes);
        let graded: Vec<EntryId> = self
            .set
            .entries()
            .iter()
            .filter(|entry| entry.msvc_name.is_some() && matches!(entry.kind, EntryKind::Function | EntryKind::Global))
            .map(|entry| entry.id.clone())
            .collect();
        let levilamina = self.levilamina.resolve_all(&graded);
        notes.extend(levilamina.notes);
        (GradeReport::compare(&self.set, &derived.found, &levilamina.found), notes)
    }

    /// Writes `report` as `<dir>/grade-<stem>.md` and returns the path.
    pub fn write(report: &GradeReport, dir: &Path, stem: &str) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(GradeReport::file_name(stem));
        std::fs::write(&path, report.markdown(stem))?;
        Ok(path)
    }
}
