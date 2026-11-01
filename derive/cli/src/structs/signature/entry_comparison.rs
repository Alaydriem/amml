use amml_data::EntryId;

use super::SignatureComparison;

/// One entry of a signature comparison. `returns` is advisory: `int`, or `int -> none` when the two builds differ,
/// and it never decides between same and changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryComparison {
    pub id: EntryId,
    pub comparison: SignatureComparison,
    pub returns: Option<String>,
}

impl EntryComparison {
    pub fn new(id: EntryId, comparison: SignatureComparison, returns: Option<String>) -> Self {
        Self { id, comparison, returns }
    }

    /// `<entry>: same | changed: ... | unknown: ...`, then `; returns (advisory): ...` when both sides are known.
    pub fn line(&self) -> String {
        match &self.returns {
            Some(returns) => format!("{}: {}; returns (advisory): {returns}", self.id, self.comparison.line()),
            None => format!("{}: {}", self.id, self.comparison.line()),
        }
    }
}
