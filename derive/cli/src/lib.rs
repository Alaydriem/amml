#![forbid(unsafe_code)]

pub mod cli;
pub mod errors;
pub mod services;
pub mod structs;
#[cfg(feature = "test-support")]
pub mod testing;

/// The version of what the deriver does with a recipe's steps. The runtime mixes it into the cache
/// name, so a change here makes every cache file stale. Bump it whenever step semantics change.
pub const DERIVER_VERSION: u32 = 11;

pub use errors::{CliError, DeriveError, StepError};
pub use services::{
    CheckRunner, CodeXref, Deriver, Disasm, EntryExplainer, FieldAccess, FixedFields, FunctionListing, FunctionMatcher,
    ImageLoader, JobWriter, LayoutGuard, PacketTable, PatternDrafter, PatternHit, PatternScanner, RecipeLint,
    RecipeRunner, ReportWriter, RttiIndex, SignatureCompare, SignatureProbe, SignatureStore, SlotNamer, StringXref,
    VtableReader,
};
pub use structs::{
    AccessKind, Address, CodeRef, CodeRefKind, CoveredInstruction, Cursor, Degraded, DeriveOutcome, EntryComparison,
    EntryLint, EntrySignature, Failure, FailureReason, FieldAccessRow, FieldKind, Fingerprint, FixedByte,
    FunctionTable, Image, Job, LayoutRefusal, LintFinding, LintReport, MatchCandidate, PacketList, PacketRow, Pattern,
    RecipeTrace, Reg, Report, ReturnKind, Section, Signature, SignatureComparison, SignatureFile, SignatureReport,
    SignatureStatus, StepTrace,
};
