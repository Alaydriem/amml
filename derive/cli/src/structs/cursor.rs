use amml_data::EntryKind;

use crate::errors::StepError;

/// The running value of a recipe, typed by what it denotes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cursor {
    Rva(u32),
    Number(u64),
    Slot(u32),
}

impl Cursor {
    /// The cursor of a resolved value of `kind`.
    pub fn of_kind(kind: EntryKind, value: u64) -> Result<Self, StepError> {
        match kind {
            EntryKind::Function | EntryKind::Global => Self::rva_of(value).map(Self::Rva),
            EntryKind::VSlot => u32::try_from(value).map(Self::Slot).map_err(|_| StepError::NotAnRva(value)),
            EntryKind::Field => Ok(Self::Number(value)),
        }
    }

    pub fn rva_of(value: u64) -> Result<u32, StepError> {
        u32::try_from(value).map_err(|_| StepError::NotAnRva(value))
    }

    pub fn slot_of(value: u64) -> Result<u32, StepError> {
        u32::try_from(value).map_err(|_| StepError::NotAnRva(value))
    }

    pub fn value(&self) -> u64 {
        match self {
            Self::Rva(v) | Self::Slot(v) => u64::from(*v),
            Self::Number(v) => *v,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Rva(_) => "rva",
            Self::Number(_) => "number",
            Self::Slot(_) => "slot",
        }
    }

    /// The RVA, or `NotAnRva` for a number or a slot.
    pub fn rva(&self) -> Result<u32, StepError> {
        match self {
            Self::Rva(v) => Ok(*v),
            other => Err(StepError::NotAnRva(other.value())),
        }
    }

    /// True when this cursor is the right type to end a recipe of `kind`.
    pub fn fits(&self, kind: EntryKind) -> bool {
        matches!(
            (kind, self),
            (EntryKind::Function | EntryKind::Global, Self::Rva(_))
                | (EntryKind::VSlot, Self::Slot(_))
                | (EntryKind::Field, Self::Number(_))
        )
    }

    /// Adds `delta`, keeping the type.
    pub fn shifted(&self, delta: i64) -> Result<Self, StepError> {
        let value = self.value();
        let overflow = StepError::Overflow { value, delta };
        let moved = value.checked_add_signed(delta).ok_or_else(|| overflow.clone())?;
        match self {
            Self::Rva(_) => u32::try_from(moved).map(Self::Rva).map_err(|_| overflow),
            Self::Slot(_) => u32::try_from(moved).map(Self::Slot).map_err(|_| overflow),
            Self::Number(_) => Ok(Self::Number(moved)),
        }
    }
}
