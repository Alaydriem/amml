use amml_data::EntryId;

use super::Signature;

/// The signature of one hooked entry; `rva` is `None` when the entry did not derive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntrySignature {
    pub id: EntryId,
    pub rva: Option<u32>,
    pub signature: Signature,
}

impl EntrySignature {
    pub fn new(id: EntryId, rva: Option<u32>, signature: Signature) -> Self {
        Self { id, rva, signature }
    }
}
