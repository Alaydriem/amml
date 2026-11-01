use amml_data::{EntryId, EntryKind};

use crate::services::ReportWriter;
use crate::structs::{FailureReason, RecipeTrace};

/// Everything a repair needs to know about one failed entry. `render` gives the text of the job file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    pub id: EntryId,
    pub kind: EntryKind,
    pub msvc_name: Option<String>,
    pub hooked: bool,
    pub reason: FailureReason,
    // the value in the earlier data file, when the run compared against one
    pub old_value: Option<u64>,
    // for a dependency failure: the first failed entry in the chain that failed for another reason
    pub root: Option<EntryId>,
    pub traces: Vec<RecipeTrace>,
    pub notes: String,
    // the recipe file relative to the recipes directory
    pub recipe_file: Option<String>,
    // the checks of the entry, one line each
    pub checks: Vec<String>,
}

impl Job {
    /// The name of the job file. Characters that a file name cannot hold become `_`.
    pub fn file_name(id: &EntryId) -> String {
        let safe: String = id.0.chars().map(|c| if c == ':' { '_' } else { c }).collect();
        format!("{safe}.md")
    }

    /// True when the entry fails for its own reason, not because an entry it needs failed.
    pub fn is_root(&self) -> bool {
        !matches!(self.reason, FailureReason::DependencyFailed(_))
    }

    /// The old value as text: `0x1234`, or `unknown` without compare data.
    pub fn old_text(&self) -> String {
        self.old_value.map_or_else(|| "unknown".to_string(), |value| format!("{value:#x}"))
    }

    pub fn render(&self) -> String {
        let mut text = format!("# {}\n\n", self.id);
        text.push_str(&format!("- id: {}\n", self.id));
        text.push_str(&format!("- kind: {}\n", self.kind.name()));
        text.push_str(&format!("- msvc_name: {}\n", self.msvc_name.as_deref().unwrap_or("none")));
        text.push_str(&format!("- hooked: {}\n", if self.hooked { "yes" } else { "no" }));
        text.push_str(&format!("- failure: {}\n", ReportWriter::reason(&self.reason)));
        text.push_str(&format!("- old value: {}\n", self.old_text()));
        match &self.root {
            Some(root) => text.push_str(&format!("- root: {root}\n")),
            None => text.push_str("- root: none (this entry fails for its own reason)\n"),
        }
        text.push_str("\n## Recipe file\n\n");
        match &self.recipe_file {
            Some(path) => text.push_str(&format!("{path}\n")),
            None => text.push_str("unknown\n"),
        }
        text.push_str("\n## Checks\n\n");
        if self.checks.is_empty() {
            text.push_str("none\n");
        } else {
            for check in &self.checks {
                text.push_str(&format!("- {check}\n"));
            }
        }
        text.push_str("\n## Notes\n\n");
        if self.notes.trim().is_empty() {
            text.push_str("none\n");
        } else {
            text.push_str(self.notes.trim_end());
            text.push('\n');
        }
        text.push_str("\n## Trace\n\n```text\n");
        text.push_str(&RecipeTrace::render(&self.traces));
        text.push_str("\n```\n");
        text.push_str("\n## Commands to start with\n\n```text\n");
        text.push_str(&format!("mise run inspect -- explain --exe <new exe> {} --recipes <recipes dir>\n", self.id));
        if let (EntryKind::Function, Some(old)) = (self.kind, self.old_value) {
            text.push_str(&format!("mise run inspect -- similar --exe <new exe> --old-exe <old exe> {old:#x}\n"));
        }
        text.push_str("```\n");
        text
    }
}
