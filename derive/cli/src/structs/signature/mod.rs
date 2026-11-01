pub mod comparison;
pub mod entry;
pub mod entry_comparison;
pub mod file;
pub mod reg;
pub mod report;
pub mod return_kind;
pub mod status;

pub use comparison::SignatureComparison;
pub use entry::EntrySignature;
pub use entry_comparison::EntryComparison;
pub use file::SignatureFile;
pub use reg::Reg;
pub use report::SignatureReport;
pub use return_kind::ReturnKind;
pub use status::SignatureStatus;

/// What a function's code shows of its calling convention: the argument registers it reads before it writes them,
/// the incoming stack slots it reads, and where it leaves a return value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    // rcx, rdx, r8 and r9, in that order, with the width of their first read
    pub args_int: Vec<Reg>,
    // xmm0 to xmm3, in that order
    pub args_xmm: Vec<Reg>,
    // offsets from the stack pointer at entry; 0x28 is the fifth argument
    pub stack_args: Vec<u32>,
    pub returns: ReturnKind,
    pub status: SignatureStatus,
}

impl Signature {
    pub fn new(args_int: Vec<Reg>, args_xmm: Vec<Reg>, stack_args: Vec<u32>, returns: ReturnKind) -> Self {
        Self { args_int, args_xmm, stack_args, returns, status: SignatureStatus::Known }
    }

    /// A signature the probe could not read; it compares as "unknown" with anything.
    pub fn unknown(reason: String) -> Self {
        Self {
            args_int: Vec::new(),
            args_xmm: Vec::new(),
            stack_args: Vec::new(),
            returns: ReturnKind::None,
            status: SignatureStatus::Unknown(reason),
        }
    }
}
