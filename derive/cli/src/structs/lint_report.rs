use amml_data::{BuildId, EntryId, EntryKind};

/// What kind of encoded field a fixed pattern byte sits in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    Immediate,
    Displacement,
    // the disp32 of a `[rip+disp32]` operand
    RipDisplacement,
    Branch,
}

impl FieldKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Immediate => "immediate",
            Self::Displacement => "displacement",
            Self::RipDisplacement => "rip displacement",
            Self::Branch => "branch",
        }
    }
}

/// One fixed pattern byte inside an immediate, displacement or branch field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedByte {
    pub recipe: usize,
    pub step: usize,
    // the alternative of a `pattern_any`; `None` for `pattern`
    pub alternative: Option<usize>,
    // counts pattern tokens from 0
    pub byte: usize,
    pub value: u8,
    // the start of the instruction that holds the byte
    pub rva: u32,
    pub field: FieldKind,
    pub instruction: String,
    // the `literal_ok` reason, when the author kept the byte
    pub kept: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LintFinding {
    Byte(FixedByte),
    // the match cannot be decoded, so the pattern is not known to be clean
    NotDecodable { recipe: usize, step: usize, rva: u32, why: String },
    // the recipe fails in the runner, so it is not known to be clean
    DidNotRun { recipe: usize, error: String },
    // a `vtable_slot` step on a `function` entry
    LiteralSlot { recipe: usize, step: usize, slot: u32 },
    // a `range` check on a `vslot` entry that allows fewer than `RecipeLint::MIN_RANGE_SLOTS` slots
    NarrowRange { min: u64, max: u64 },
}

impl LintFinding {
    /// True when the finding counts against the exit code; a kept byte does not.
    pub fn is_open(&self) -> bool {
        !matches!(self, Self::Byte(FixedByte { kept: Some(_), .. }))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryLint {
    pub id: EntryId,
    pub kind: EntryKind,
    pub findings: Vec<LintFinding>,
    // facts that never fail the run: unused `literal_ok` entries and `pattern_any` alternatives that were not linted
    pub info: Vec<String>,
    // set when an `entry_is` guard has another value on this build: the entry is not linted
    pub not_for_build: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintReport {
    pub build: BuildId,
    // every entry of the recipe set, in set order
    pub entries: Vec<EntryLint>,
}

impl LintReport {
    pub fn open_findings(&self) -> usize {
        self.entries.iter().flat_map(|entry| &entry.findings).filter(|finding| finding.is_open()).count()
    }

    /// Entries skipped because an `entry_is` guard refuses them on this build.
    pub fn skipped(&self) -> usize {
        self.entries.iter().filter(|entry| entry.not_for_build.is_some()).count()
    }

    pub fn kept_bytes(&self) -> usize {
        self.entries.iter().flat_map(|entry| &entry.findings).filter(|finding| !finding.is_open()).count()
    }

    /// 0 when nothing is open, 1 otherwise.
    pub fn exit_code(&self) -> u8 {
        u8::from(self.open_findings() > 0)
    }

    pub fn summary_line(&self) -> String {
        format!(
            "lint: {} entries, {} open findings, {} kept bytes, {} not for this build",
            self.entries.len(),
            self.open_findings(),
            self.kept_bytes(),
            self.skipped()
        )
    }
}
