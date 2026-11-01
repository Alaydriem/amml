use crate::structs::EntryKind;

/// What a running recipe value denotes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    Rva,
    Number,
    Slot,
}

impl ValueType {
    /// The type a recipe of an entry of `kind` has to end in.
    pub fn of_kind(kind: EntryKind) -> Self {
        match kind {
            EntryKind::Function | EntryKind::Global => Self::Rva,
            EntryKind::VSlot => Self::Slot,
            EntryKind::Field => Self::Number,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Rva => "rva",
            Self::Number => "number",
            Self::Slot => "slot",
        }
    }
}
