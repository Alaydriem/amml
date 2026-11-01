pub mod covered_instruction;
pub mod cursor;
pub mod image;
pub mod inspect;
pub mod job;
pub mod lint_report;
pub mod outcome;
pub mod pattern;
pub mod report;
pub mod signature;

pub use covered_instruction::CoveredInstruction;
pub use cursor::Cursor;
pub use image::{FunctionTable, Image, Section};
pub(crate) use inspect::FunctionIndex;
pub use inspect::{
    AccessKind, Address, CodeRef, CodeRefKind, FieldAccessRow, Fingerprint, MatchCandidate, PacketList, PacketRow,
    RecipeTrace, StepTrace,
};
pub use job::Job;
pub use lint_report::{EntryLint, FieldKind, FixedByte, LintFinding, LintReport};
pub use outcome::{Degraded, DeriveOutcome, Failure, FailureReason, LayoutRefusal};
pub use pattern::Pattern;
pub use report::Report;
pub use signature::{
    EntryComparison, EntrySignature, Reg, ReturnKind, Signature, SignatureComparison, SignatureFile, SignatureReport,
    SignatureStatus,
};
