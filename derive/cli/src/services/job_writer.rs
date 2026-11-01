use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use amml_data::{BuildData, DependencyOrder, EntryId, RecipeSet};

use crate::services::{Deriver, EntryExplainer, ReportWriter};
use crate::structs::{Degraded, DeriveOutcome, FailureReason, Image, Job};

// a bound for the walk along a dependency chain; the chain of a derived set never loops
const MAX_CHAIN: usize = 100_000;

pub struct JobWriter<'a> {
    image: &'a Image,
    set: &'a RecipeSet,
    outcome: &'a DeriveOutcome,
    old: Option<&'a BuildData>,
}

impl<'a> JobWriter<'a> {
    /// The folder of the job files inside the build folder. The index stays beside it, so no id can overwrite it.
    pub const JOBS_DIR: &'static str = "jobs";

    pub fn new(image: &'a Image, set: &'a RecipeSet, outcome: &'a DeriveOutcome, old: Option<&'a BuildData>) -> Self {
        Self { image, set, outcome, old }
    }

    /// Writes `<dir>/<build>/README.md` and one `jobs/<id>.md` per failed entry. The recipes are traced against the
    /// outcome that is already known, so the set is not derived again. Returns the job files, then the index.
    pub fn write(&self, dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
        let out = dir.join(self.outcome.data.build.file_stem());
        let jobs_dir = out.join(Self::JOBS_DIR);
        fs::create_dir_all(&jobs_dir)?;
        let jobs = self.jobs()?;
        let mut paths = Vec::with_capacity(jobs.len() + 1);
        for job in &jobs {
            let path = jobs_dir.join(Job::file_name(&job.id));
            fs::write(&path, job.render())?;
            paths.push(path);
        }
        let index = out.join("README.md");
        fs::write(&index, self.index(&jobs))?;
        paths.push(index);
        Ok(paths)
    }

    /// One job per failure: the roots first, then the dependents, each group in recipe dependency order.
    pub fn jobs(&self) -> Result<Vec<Job>, io::Error> {
        if self.outcome.failures.is_empty() {
            return Ok(Vec::new());
        }
        let reasons: BTreeMap<&EntryId, &FailureReason> =
            self.outcome.failures.iter().map(|failure| (&failure.id, &failure.reason)).collect();
        let deriver = Deriver::new(self.image, self.set);
        let explainer = EntryExplainer::new(self.image, self.set);
        let mut jobs = Vec::with_capacity(self.outcome.failures.len());
        for failure in &self.outcome.failures {
            let entry = self
                .set
                .get(&failure.id)
                .ok_or_else(|| io::Error::other(format!("failed entry {} is not in the recipe set", failure.id)))?;
            let traces = explainer
                .trace(&failure.id, &deriver, self.outcome)
                .map_err(|error| io::Error::other(error.to_string()))?;
            jobs.push(Job {
                id: failure.id.clone(),
                kind: entry.kind,
                msvc_name: entry.msvc_name.clone(),
                hooked: entry.hooked,
                reason: failure.reason.clone(),
                old_value: self.old.and_then(|old| old.entries.get(&failure.id)).map(|resolved| resolved.value),
                root: Self::root_of(&failure.reason, &reasons),
                traces,
                notes: entry.notes.clone(),
                recipe_file: self.set.relative_source_of(&failure.id),
                checks: entry.checks.iter().map(ReportWriter::check).collect(),
            });
        }
        let rank = self.rank();
        jobs.sort_by_key(|job| (!job.is_root(), rank.get(&job.id).copied().unwrap_or(usize::MAX), job.id.clone()));
        Ok(jobs)
    }

    // the first failed entry along the chain that failed for a reason other than a dependency
    fn root_of(reason: &FailureReason, reasons: &BTreeMap<&EntryId, &FailureReason>) -> Option<EntryId> {
        let FailureReason::DependencyFailed(first) = reason else { return None };
        let mut current = first;
        for _ in 0..MAX_CHAIN {
            match reasons.get(current) {
                Some(FailureReason::DependencyFailed(next)) => current = next,
                _ => break,
            }
        }
        Some(current.clone())
    }

    // the position of each entry in the recipe dependency order; empty when the set has a cycle
    fn rank(&self) -> BTreeMap<EntryId, usize> {
        DependencyOrder::of(self.set)
            .map(|order| order.into_iter().enumerate().map(|(index, id)| (id, index)).collect())
            .unwrap_or_default()
    }

    fn index(&self, jobs: &[Job]) -> String {
        let roots = jobs.iter().filter(|job| job.is_root()).count();
        let degraded = self.degraded();
        let mut text = format!("# Repair jobs for {}\n\n", self.outcome.data.build.file_stem());
        text.push_str(&format!("- failed: {} ({} roots, {} dependents)\n", jobs.len(), roots, jobs.len() - roots));
        text.push_str(&format!("- degraded: {}\n", degraded.len()));
        text.push_str("\nRepair the roots first. A dependent usually passes once its root is repaired.\n");
        text.push_str("\n## Failed entries\n\n");
        if jobs.is_empty() {
            text.push_str("none\n");
        } else {
            text.push_str("| id | kind | root | failure | old value | job |\n| --- | --- | --- | --- | --- | --- |\n");
            for job in jobs {
                let root = job.root.as_ref().map_or_else(|| "-".to_string(), |root| root.0.clone());
                let file = format!("{}/{}", Self::JOBS_DIR, Job::file_name(&job.id));
                text.push_str(&format!(
                    "| {} | {} | {} | {} | {} | [{}]({}) |\n",
                    Self::cell(&job.id.0),
                    job.kind.name(),
                    Self::cell(&root),
                    Self::cell(&ReportWriter::reason(&job.reason)),
                    job.old_text(),
                    Self::cell(&file),
                    file
                ));
            }
        }
        text.push_str("\n## Degraded\n\n");
        text.push_str("Resolved, but a recipe of the entry failed. Low priority: the value is derived.\n\n");
        if degraded.is_empty() {
            text.push_str("none\n");
        } else {
            text.push_str("| id | recipe | error |\n| --- | --- | --- |\n");
            for entry in degraded {
                for (index, error) in &entry.recipe_errors {
                    text.push_str(&format!(
                        "| {} | {} | {} |\n",
                        Self::cell(&entry.id.0),
                        index,
                        Self::cell(&error.to_string())
                    ));
                }
            }
        }
        text
    }

    // resolved entries with at least one failed recipe, in dependency order
    fn degraded(&self) -> Vec<&Degraded> {
        self.outcome.degraded.iter().filter(|entry| !entry.recipe_errors.is_empty()).collect()
    }

    fn cell(text: &str) -> String {
        text.replace('|', "\\|")
    }
}
