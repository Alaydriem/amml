use std::collections::BTreeMap;
use std::time::Duration;

use amml_data::{BuildData, Check, EntryId, RecipeSet};

use crate::services::Deriver;
use crate::structs::{DeriveOutcome, FailureReason, Report};

pub struct ReportWriter;

impl ReportWriter {
    // builds the report that `render` prints; the constructor name is part of the writer's interface
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        outcome: &DeriveOutcome,
        set: &RecipeSet,
        elapsed: Duration,
        deterministic: bool,
        previous: Option<&BuildData>,
    ) -> Report {
        let mut failures = outcome.failures.clone();
        failures.sort_by(|a, b| a.id.cmp(&b.id));
        let mut degraded = outcome.degraded.clone();
        degraded.sort_by(|a, b| a.id.cmp(&b.id));
        let notes: BTreeMap<EntryId, String> = set
            .entries()
            .iter()
            .filter(|entry| !entry.notes.is_empty())
            .map(|entry| (entry.id.clone(), entry.notes.clone()))
            .collect();
        let stored_failures: Vec<(EntryId, String)> = outcome
            .reused
            .iter()
            .filter_map(|id| outcome.data.failed.get(id).map(|reason| (id.clone(), reason.clone())))
            .collect();
        let mut reused_ids = outcome.reused.clone();
        reused_ids.sort();
        let moved = previous.map(|old| Self::moved(old, &outcome.data)).unwrap_or_default();
        Report {
            build: outcome.data.build,
            total: set.entries().len(),
            derived: outcome.data.entries.len(),
            reused: outcome.reused.len(),
            reused_ids,
            stored_failures,
            failures,
            degraded,
            notes,
            elapsed,
            deterministic,
            moved,
        }
    }

    pub fn render(report: &Report) -> String {
        let mut text = String::new();
        text.push_str("# AMML derive report\n\n");
        text.push_str(&format!("- build: {}\n", report.build.file_stem()));
        text.push_str(&format!("- derived {}/{} ({:.2}%)\n", report.derived, report.total, report.percent()));
        text.push_str(&format!(
            "- target: {:.0}% ({})\n",
            Report::target_percent(),
            if report.target_met() { "met" } else { "not met" }
        ));
        if report.deterministic {
            text.push_str("- deterministic: yes\n");
        } else {
            text.push_str("- deterministic: no (the two runs differ, so no data file was written)\n");
        }
        text.push_str(&format!("- reused {}, derived {}\n", report.reused, report.total.saturating_sub(report.reused)));
        text.push_str(&format!("- elapsed: {:.1}s\n", report.elapsed.as_secs_f64()));
        text.push_str("\n## Failures\n\n");
        if report.failures.is_empty() && report.stored_failures.is_empty() {
            text.push_str("none\n");
        } else {
            text.push_str("| id | reason | notes |\n| --- | --- | --- |\n");
            for failure in &report.failures {
                let note = report.notes.get(&failure.id).and_then(|notes| notes.lines().next()).unwrap_or("");
                text.push_str(&format!(
                    "| {} | {} | {} |\n",
                    Self::cell(&failure.id.0),
                    Self::cell(&Self::reason(&failure.reason)),
                    Self::cell(note)
                ));
            }
            for (id, reason) in &report.stored_failures {
                let note = report.notes.get(id).and_then(|notes| notes.lines().next()).unwrap_or("");
                text.push_str(&format!(
                    "| {} | {} | {} |\n",
                    Self::cell(&id.0),
                    Self::cell(&format!("{reason} (stored)")),
                    Self::cell(note)
                ));
            }
        }
        Self::render_degraded(report, &mut text);
        if !report.reused_ids.is_empty() {
            text.push_str("\n## Reused\n\n");
            for id in &report.reused_ids {
                text.push_str(&format!("- {id}\n"));
            }
        }
        if !report.moved.is_empty() {
            text.push_str("\n## Moved\n\n");
            text.push_str("| id | old | new |\n| --- | --- | --- |\n");
            for (id, old, new) in &report.moved {
                text.push_str(&format!("| {} | {:#x} | {:#x} |\n", Self::cell(&id.0), old, new));
            }
        }
        text
    }

    // entries that resolved with a failed recipe, then entries that resolved on one unchecked recipe
    fn render_degraded(report: &Report, text: &mut String) {
        text.push_str("\n## Degraded\n\n");
        if report.degraded.iter().all(|entry| entry.recipe_errors.is_empty()) {
            text.push_str("none\n");
        } else {
            text.push_str("| id | recipe | error |\n| --- | --- | --- |\n");
            for entry in &report.degraded {
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
        text.push_str("\n## Unverified\n\n");
        if report.degraded.iter().all(|entry| !entry.unverified) {
            text.push_str("none\n");
        } else {
            text.push_str("Resolved by a single recipe and no checks:\n\n");
            for entry in report.degraded.iter().filter(|entry| entry.unverified) {
                text.push_str(&format!("- {}\n", entry.id));
            }
        }
    }

    fn moved(old: &BuildData, new: &BuildData) -> Vec<(EntryId, u64, u64)> {
        new.entries
            .iter()
            .filter_map(|(id, now)| {
                let before = old.entries.get(id)?;
                (before.value != now.value).then(|| (id.clone(), before.value, now.value))
            })
            .collect()
    }

    /// The failure reason as one line of text.
    pub fn reason(reason: &FailureReason) -> String {
        match reason {
            FailureReason::AllRecipesFailed(errors) => {
                let list: Vec<String> = errors.iter().map(|error| error.to_string()).collect();
                format!("all recipes failed: {}", list.join("; "))
            }
            FailureReason::Disagree(values) => {
                let list: Vec<String> = values.iter().map(|value| format!("{value:#x}")).collect();
                format!("recipes disagree: {}", list.join(", "))
            }
            FailureReason::CheckFailed(check) => format!("check failed: {}", Self::check(check)),
            FailureReason::DependencyFailed(target) => format!("dependency failed: {target}"),
            FailureReason::VtableRouteOnly => "only vtable-slot recipes succeeded; none confirms the slot".to_string(),
            FailureReason::WeakChecksOnly { agreeing } => format!(
                "only section or range checks confirm this entry, so it needs {} agreeing recipes; {agreeing} agreed",
                Deriver::WEAK_CHECKS_ONLY_RECIPES
            ),
        }
    }

    /// The check as one line of text.
    pub fn check(check: &Check) -> String {
        match check {
            Check::FunctionStart => "function_start".to_string(),
            Check::LeafStart => "leaf_start".to_string(),
            Check::InVtableOf { class } => format!("in_vtable_of {class}"),
            Check::InVtable { vtable, min_slot, max_slot } => format!("in_vtable {vtable} {min_slot}..={max_slot}"),
            Check::Section { name } => format!("section {name}"),
            Check::Range { min, max } => format!("range {min:#x}..{max:#x}"),
            Check::EntryIs { entry, value } => format!("entry_is {entry} = {value:#x}"),
        }
    }

    fn cell(text: &str) -> String {
        text.replace('|', "\\|")
    }
}
