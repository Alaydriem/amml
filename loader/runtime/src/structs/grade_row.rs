use amml_data::EntryId;

use super::GradeVerdict;

/// One graded entry: an entry with an `msvc_name` that LeviLamina can look up, and the RVA each side gave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GradeRow {
    pub id: EntryId,
    pub msvc_name: String,
    pub derived: Option<u64>,
    pub levilamina: Option<u64>,
}

impl GradeRow {
    pub fn verdict(&self) -> GradeVerdict {
        match (self.derived, self.levilamina) {
            (Some(derived), Some(levilamina)) if derived == levilamina => GradeVerdict::Match,
            (Some(_), Some(_)) => GradeVerdict::Mismatch,
            (Some(_), None) => GradeVerdict::DerivedOnly,
            (None, Some(_)) => GradeVerdict::LeviLaminaOnly,
            (None, None) => GradeVerdict::Neither,
        }
    }
}
