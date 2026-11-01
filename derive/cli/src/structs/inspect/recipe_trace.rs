use super::StepTrace;
use crate::errors::StepError;

/// One recipe of an entry, run step by step. The steps stop at the first one that fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeTrace {
    pub index: usize,
    pub steps: Vec<StepTrace>,
    pub result: Result<u64, StepError>,
}

impl RecipeTrace {
    /// One line per step, then the result of the recipe.
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self.steps.iter().map(|step| step.line(self.index)).collect();
        lines.push(match &self.result {
            Ok(value) => format!("recipe {} result {value:#x}", self.index),
            Err(error) => format!("recipe {} failed: {error}", self.index),
        });
        lines
    }

    /// The lines of every trace, joined with newlines. The `inspect explain` output and the repair jobs use it.
    pub fn render(traces: &[RecipeTrace]) -> String {
        traces.iter().flat_map(RecipeTrace::lines).collect::<Vec<_>>().join("\n")
    }
}
