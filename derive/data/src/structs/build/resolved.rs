use crate::structs::EntryKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub kind: EntryKind,
    pub value: u64,
    pub recipes_agreeing: u32,
}
