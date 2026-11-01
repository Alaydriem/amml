/// How a graded entry compares with LeviLamina's address for the same `msvc_name`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GradeVerdict {
    Match,
    Mismatch,
    DerivedOnly,
    LeviLaminaOnly,
    Neither,
}

impl GradeVerdict {
    pub const ALL: [GradeVerdict; 5] =
        [Self::Match, Self::Mismatch, Self::DerivedOnly, Self::LeviLaminaOnly, Self::Neither];

    pub fn label(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Mismatch => "mismatch",
            Self::DerivedOnly => "derived-only",
            Self::LeviLaminaOnly => "levilamina-only",
            Self::Neither => "neither",
        }
    }
}
