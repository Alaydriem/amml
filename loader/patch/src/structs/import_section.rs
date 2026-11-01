use crate::errors::PatchError;
use crate::structs::ImportDescriptor;

/// The bytes of the new section and the size of the import directory inside them.
pub struct ImportSection {
    pub bytes: Vec<u8>,
    pub directory_size: u32,
}

impl ImportSection {
    // Layout: descriptors, ILT, IAT, hint/name entry, DLL name.
    pub fn build(
        existing: &[ImportDescriptor],
        virtual_address: u32,
        dll: &str,
        symbol: &str,
    ) -> Result<Self, PatchError> {
        let directory_size = (existing.len() + 2) * ImportDescriptor::SIZE;
        let ilt = directory_size.next_multiple_of(8);
        let iat = ilt + 16;
        let hint_name = iat + 16;
        let name = hint_name + (2 + symbol.len() + 1).next_multiple_of(2);
        let total = name + dll.len() + 1;
        let rva = |offset: usize| -> Result<u32, PatchError> {
            u32::try_from(offset)
                .ok()
                .and_then(|offset| virtual_address.checked_add(offset))
                .ok_or_else(|| PatchError::TooLarge("the new section runs past 32 bits".to_string()))
        };
        let mut bytes = vec![0u8; total];
        for (index, descriptor) in existing.iter().enumerate() {
            let at = index * ImportDescriptor::SIZE;
            bytes[at..at + ImportDescriptor::SIZE].copy_from_slice(&descriptor.raw);
        }
        let added = ImportDescriptor::new(rva(ilt)?, rva(name)?, rva(iat)?);
        let at = existing.len() * ImportDescriptor::SIZE;
        bytes[at..at + ImportDescriptor::SIZE].copy_from_slice(&added.raw);
        let thunk = u64::from(rva(hint_name)?).to_le_bytes();
        bytes[ilt..ilt + 8].copy_from_slice(&thunk);
        bytes[iat..iat + 8].copy_from_slice(&thunk);
        bytes[hint_name + 2..hint_name + 2 + symbol.len()].copy_from_slice(symbol.as_bytes());
        bytes[name..name + dll.len()].copy_from_slice(dll.as_bytes());
        let directory_size = u32::try_from(directory_size)
            .map_err(|_| PatchError::TooLarge("the import directory runs past 32 bits".to_string()))?;
        Ok(Self { bytes, directory_size })
    }
}
