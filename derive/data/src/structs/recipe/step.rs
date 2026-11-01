use super::{LiteralOk, OperandKind, ValueType};
use crate::structs::EntryId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    // `literal_ok` only informs `amml-derive lint`; the deriver never reads it
    Pattern { bytes: String, offset: i64, literal_ok: Vec<LiteralOk> },
    // like `pattern` with several alternatives; all hits together must be exactly one distinct position
    PatternAny { patterns: Vec<String>, offset: i64, literal_ok: Vec<LiteralOk> },
    // the primary start of the function holding the cursor
    FunctionStartOf,
    // `nth` absent: exactly one function must load the string
    StringXref { text: String, nth: Option<u32> },
    RttiVtable { class: String, slot: Option<u32> },
    Ref { entry: EntryId },
    Call { nth: u32 },
    Operand { index: u32, kind: OperandKind },
    SlotIn { class: String },
    // reads slots `0..=slot` of the vtable at the cursor; every one must point into code
    VtableSlot { slot: u32 },
    // reads the qword at the cursor; minus the image base it must point into code
    ReadPointer,
    // the slot that holds the function at the cursor, in the vtable of the named `global` entry
    SlotOf { vtable: EntryId },
    // a byte displacement `n` becomes slot `n / 8`
    DisplacementToSlot,
    Add { value: i64 },
    // the member offset read by memory operand `index` of the instruction at the cursor. When the base register is
    // a plain object pointer this is its displacement (as `operand` displacement). When the base is the frame
    // (rbp or rsp) the object is a local, and the code names a member only as `[rbp-18h]` next to the `lea` of the
    // local (`[rbp-58h]`): the offset is the member displacement minus the displacement of memory operand 1 of the
    // instruction `local_at` bytes from the cursor (it may be negative). An immediate operand (`add rsi,30h`, the
    // address of a member formed from the object pointer) gives the immediate. The result must not be negative
    MemberDisplacement { index: u32, local_at: i64 },
    // the index of the `pattern_any` alternative that matched in the step right before, as a number; fails when
    // more than one alternative matched that spot
    Alternative,
}

impl Step {
    // True for steps that produce an address without a previous value.
    pub fn is_source(&self) -> bool {
        matches!(
            self,
            Self::Pattern { .. }
                | Self::PatternAny { .. }
                | Self::StringXref { .. }
                | Self::RttiVtable { .. }
                | Self::Ref { .. }
        )
    }

    // True for steps allowed after the first one: transforms, and `pattern` / `pattern_any` which scan from the current address.
    pub fn may_follow(&self) -> bool {
        !self.is_source() || matches!(self, Self::Pattern { .. } | Self::PatternAny { .. })
    }

    pub fn referenced_entry(&self) -> Option<&EntryId> {
        match self {
            Self::Ref { entry } | Self::SlotOf { vtable: entry } => Some(entry),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Pattern { .. } => "pattern",
            Self::PatternAny { .. } => "pattern_any",
            Self::FunctionStartOf => "function_start_of",
            Self::StringXref { .. } => "string_xref",
            Self::RttiVtable { .. } => "rtti_vtable",
            Self::Ref { .. } => "ref",
            Self::Call { .. } => "call",
            Self::Operand { .. } => "operand",
            Self::SlotIn { .. } => "slot_in",
            Self::VtableSlot { .. } => "vtable_slot",
            Self::ReadPointer => "read_pointer",
            Self::SlotOf { .. } => "slot_of",
            Self::DisplacementToSlot => "displacement_to_slot",
            Self::Add { .. } => "add",
            Self::MemberDisplacement { .. } => "member_displacement",
            Self::Alternative => "alternative",
        }
    }

    /// The type of the running value after this step, given the type before it (`None` for the first step)
    /// and, for `ref`, the type of the referenced entry. Fails with a message when the step cannot take `input`.
    pub fn result_type(&self, input: Option<ValueType>, referenced: Option<ValueType>) -> Result<ValueType, String> {
        let needs_rva = || match input {
            Some(ValueType::Rva) => Ok(()),
            Some(other) => Err(format!("step `{}` needs an rva but gets a {}", self.name(), other.label())),
            None => Err(format!("step `{}` has no value to work on", self.name())),
        };
        match self {
            Self::Pattern { .. } | Self::PatternAny { .. } => match input {
                None => Ok(ValueType::Rva),
                Some(_) => needs_rva().map(|()| ValueType::Rva),
            },
            Self::StringXref { .. } | Self::RttiVtable { .. } => Ok(ValueType::Rva),
            Self::Ref { entry } => referenced.ok_or_else(|| format!("step `ref` names unknown entry {entry}")),
            Self::Call { .. } | Self::FunctionStartOf => needs_rva().map(|()| ValueType::Rva),
            Self::Operand { kind, .. } => needs_rva().map(|()| match kind {
                OperandKind::Displacement | OperandKind::Immediate | OperandKind::ScaleFactor => ValueType::Number,
                OperandKind::RipTarget | OperandKind::BranchTarget => ValueType::Rva,
            }),
            Self::SlotIn { .. } | Self::SlotOf { .. } => needs_rva().map(|()| ValueType::Slot),
            Self::VtableSlot { .. } | Self::ReadPointer => needs_rva().map(|()| ValueType::Rva),
            Self::DisplacementToSlot => match input {
                Some(ValueType::Number) => Ok(ValueType::Slot),
                Some(other) => Err(format!("step `{}` needs a number but gets a {}", self.name(), other.label())),
                None => Err(format!("step `{}` has no value to work on", self.name())),
            },
            Self::Add { .. } => input.ok_or_else(|| format!("step `{}` has no value to work on", self.name())),
            Self::Alternative | Self::MemberDisplacement { .. } => needs_rva().map(|()| ValueType::Number),
        }
    }
}
