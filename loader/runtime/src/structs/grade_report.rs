use std::collections::BTreeMap;
use std::fmt::Write as _;

use amml_data::{EntryId, EntryKind, RecipeSet, Resolved};

use super::{GradeRow, GradeVerdict};

/// AMML's derived addresses measured against LeviLamina's for the same `msvc_name`s.
///
/// A measurement only: nothing in it feeds the entry table. LeviLamina may know only some of the
/// names, so every row is graded on its own and the report counts the rows that have a reference.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GradeReport {
    /// Every `function` and `global` entry with an `msvc_name`, in id order.
    pub rows: Vec<GradeRow>,
}

impl GradeReport {
    /// Compares the derived values (cache or derive) with LeviLamina's for every `function` and
    /// `global` entry of `set` that has an `msvc_name`. Both maps hold RVAs.
    pub fn compare(
        set: &RecipeSet,
        derived: &BTreeMap<EntryId, Resolved>,
        levilamina: &BTreeMap<EntryId, Resolved>,
    ) -> Self {
        let mut rows: Vec<GradeRow> = set
            .entries()
            .iter()
            .filter(|entry| matches!(entry.kind, EntryKind::Function | EntryKind::Global))
            .filter_map(|entry| {
                let msvc_name = entry.msvc_name.clone()?;
                Some(GradeRow {
                    id: entry.id.clone(),
                    msvc_name,
                    derived: derived.get(&entry.id).map(|resolved| resolved.value),
                    levilamina: levilamina.get(&entry.id).map(|resolved| resolved.value),
                })
            })
            .collect();
        rows.sort_by(|a, b| a.id.cmp(&b.id));
        Self { rows }
    }

    /// How many rows have `verdict`.
    pub fn count(&self, verdict: GradeVerdict) -> usize {
        self.rows.iter().filter(|row| row.verdict() == verdict).count()
    }

    /// How many names LeviLamina gave a usable address for: the rows that have a reference.
    pub fn reference_count(&self) -> usize {
        self.rows.iter().filter(|row| row.levilamina.is_some()).count()
    }

    pub fn mismatches(&self) -> impl Iterator<Item = &GradeRow> {
        self.rows.iter().filter(|row| row.verdict() == GradeVerdict::Mismatch)
    }

    /// `grade: reference for N of M names; match A, mismatch B, derived-only C, levilamina-only D,
    /// neither E`. A name without a reference is `derived-only` or `neither`.
    pub fn summary_line(&self) -> String {
        let counts: Vec<String> =
            GradeVerdict::ALL.iter().map(|verdict| format!("{} {}", verdict.label(), self.count(*verdict))).collect();
        format!("grade: reference for {} of {} names; {}", self.reference_count(), self.rows.len(), counts.join(", "))
    }

    /// `grade-<stem>.md`.
    pub fn file_name(stem: &str) -> String {
        format!("grade-{stem}.md")
    }

    /// The report file: the reference coverage, the counts, the mismatch table and every graded entry.
    pub fn markdown(&self, stem: &str) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "# AMML grade report
"
        );
        let _ = writeln!(text, "- build: {stem}");
        let _ = writeln!(text, "- reference: {} of {} names", self.reference_count(), self.rows.len());
        for verdict in GradeVerdict::ALL {
            let _ = writeln!(text, "- {}: {}", verdict.label(), self.count(verdict));
        }
        let _ = writeln!(text, "- graded: {}\n", self.rows.len());
        let _ = writeln!(text, "## Mismatches\n");
        if self.mismatches().next().is_none() {
            let _ = writeln!(text, "none\n");
        } else {
            let _ = writeln!(text, "| id | msvc_name | derived | levilamina |\n| --- | --- | --- | --- |");
            for row in self.mismatches() {
                let _ = writeln!(
                    text,
                    "| {} | `{}` | {} | {} |",
                    row.id,
                    row.msvc_name,
                    Self::hex(row.derived),
                    Self::hex(row.levilamina)
                );
            }
            text.push('\n');
        }
        let _ = writeln!(text, "## Entries\n");
        let _ = writeln!(text, "| id | msvc_name | derived | levilamina | verdict |\n| --- | --- | --- | --- | --- |");
        for row in &self.rows {
            let _ = writeln!(
                text,
                "| {} | `{}` | {} | {} | {} |",
                row.id,
                row.msvc_name,
                Self::hex(row.derived),
                Self::hex(row.levilamina),
                row.verdict().label()
            );
        }
        text
    }

    fn hex(value: Option<u64>) -> String {
        value.map_or_else(|| "-".to_string(), |value| format!("{value:#x}"))
    }
}
