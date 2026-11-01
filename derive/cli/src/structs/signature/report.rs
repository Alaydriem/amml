use amml_data::BuildId;

use super::{EntryComparison, SignatureComparison};

/// The comparison of two signature files, entry by entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureReport {
    pub old: BuildId,
    pub new: BuildId,
    pub entries: Vec<EntryComparison>,
}

impl SignatureReport {
    pub fn new(old: BuildId, new: BuildId, entries: Vec<EntryComparison>) -> Self {
        Self { old, new, entries }
    }

    pub fn count(&self, matches: fn(&SignatureComparison) -> bool) -> usize {
        self.entries.iter().filter(|entry| matches(&entry.comparison)).count()
    }

    /// 0 when every entry is the same; 1 when any entry changed or could not be compared, because an unknown
    /// signature is not known to be safe.
    pub fn exit_code(&self) -> u8 {
        let open = self.count(|comparison| !matches!(comparison, SignatureComparison::Same));
        u8::from(open > 0)
    }

    /// One line per entry.
    pub fn render(&self) -> String {
        self.entries.iter().map(|entry| format!("{}\n", entry.line())).collect()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "signatures {} against {}: {} same, {} changed, {} unknown",
            self.new.file_stem(),
            self.old.file_stem(),
            self.count(|comparison| matches!(comparison, SignatureComparison::Same)),
            self.count(|comparison| matches!(comparison, SignatureComparison::Changed(_))),
            self.count(|comparison| matches!(comparison, SignatureComparison::Unknown(_)))
        )
    }
}
