use super::OriginalValue;

/// One `original` slot write the engine must apply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalWrite {
    /// Address of the slot, owned by the mod that registered the hook.
    pub slot: u64,
    pub value: OriginalValue,
}
