use std::collections::BTreeMap;
use std::time::Duration;

use amml_data::{BuildId, EntryId};

use super::{Degraded, Failure};

// the share of entries a derive run should reach
const TARGET_PERCENT: f64 = 95.0;

#[derive(Clone, Debug)]
pub struct Report {
    pub build: BuildId,
    pub total: usize,
    pub derived: usize,
    // sorted by entry id
    pub failures: Vec<Failure>,
    // resolved entries with a failed recipe or a single unchecked recipe, sorted by entry id
    pub degraded: Vec<Degraded>,
    pub notes: BTreeMap<EntryId, String>,
    pub elapsed: Duration,
    pub deterministic: bool,
    // entries in the previous data file and in this run whose value changed: id, old, new
    pub moved: Vec<(EntryId, u64, u64)>,
}

impl Report {
    // exit code of a run whose two derives differ
    pub const NOT_DETERMINISTIC_EXIT: u8 = 3;

    pub fn percent(&self) -> f64 {
        if self.total == 0 { 0.0 } else { self.derived as f64 * 100.0 / self.total as f64 }
    }

    pub fn target_met(&self) -> bool {
        self.percent() >= TARGET_PERCENT
    }

    pub fn target_percent() -> f64 {
        TARGET_PERCENT
    }

    /// 0 for a deterministic run, `NOT_DETERMINISTIC_EXIT` otherwise.
    pub fn exit_code(&self) -> u8 {
        if self.deterministic { 0 } else { Self::NOT_DETERMINISTIC_EXIT }
    }

    pub fn summary_line(&self) -> String {
        let determinism = if self.deterministic { "deterministic" } else { "not deterministic" };
        format!(
            "derived {}/{} ({:.2}%) in {:.1}s, {}",
            self.derived,
            self.total,
            self.percent(),
            self.elapsed.as_secs_f64(),
            determinism
        )
    }
}
