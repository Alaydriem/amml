use amml_data::{EntryId, EntryKind};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum StepError {
    #[error("no match")]
    NoMatch,
    #[error("{0} matches where one was expected")]
    ManyMatches(usize),
    #[error("cannot decode an instruction at rva {0:#x}")]
    BadInstruction(u32),
    #[error("instruction at rva {rva:#x} has no usable operand {index}")]
    NoOperand { rva: u32, index: u32 },
    #[error("rva {0:#x} is not inside a function")]
    NotInFunction(u32),
    #[error("rva {0:#x} is not in a function")]
    NotInListedFunction(u32),
    #[error("function {0:#x} has no strings, imports or large constants: nothing to match on")]
    NothingToMatch(u32),
    #[error("rva {0:#x} is not an instruction start")]
    NotInstructionStart(u32),
    #[error("{0}")]
    Undecodable(String),
    #[error("call {nth} requested but the function has {found}")]
    CallIndex { nth: u32, found: u32 },
    #[error("class `{0}` not found")]
    ClassNotFound(String),
    #[error("class `{0}` is ambiguous")]
    ClassAmbiguous(String),
    #[error("slot {slot} is outside the vtable of `{class}`")]
    SlotOutOfRange { class: String, slot: u32 },
    #[error("rva {rva:#x} is not in a vtable of `{class}`")]
    NotInVtable { class: String, rva: u32 },
    #[error("rva {rva:#x} fills more than one slot of `{class}`")]
    AmbiguousSlot { class: String, rva: u32 },
    #[error("string `{0}` not found")]
    StringNotFound(String),
    #[error("reference {nth} requested but the string has {found}")]
    XrefIndex { nth: u32, found: u32 },
    #[error("value {0:#x} is not an rva")]
    NotAnRva(u64),
    #[error("the pointer at rva {rva:#x} does not point into code")]
    NotCode { rva: u32 },
    #[error("displacement {0:#x} is not a multiple of 8")]
    NotSlotAligned(u64),
    #[error("{value:#x} shifted by {delta} overflows")]
    Overflow { value: u64, delta: i64 },
    #[error("a `{}` entry cannot end in {got}", kind.name())]
    WrongResult { kind: EntryKind, got: String },
    #[error("bad pattern: {0}")]
    BadPattern(String),
    #[error("dependency `{0}` failed")]
    DependencyFailed(EntryId),
    #[error("no `pattern_any` match right before `alternative`")]
    NoAlternative,
    #[error("alternatives {0:?} matched the same spot, so `alternative` cannot tell them apart")]
    AmbiguousAlternative(Vec<usize>),
}
