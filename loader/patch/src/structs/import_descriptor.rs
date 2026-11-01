use crate::errors::PatchError;
use crate::structs::ByteView;

/// One 20-byte `IMAGE_IMPORT_DESCRIPTOR`, kept as the bytes found in the file.
#[derive(Clone, Copy, Debug)]
pub struct ImportDescriptor {
    pub raw: [u8; ImportDescriptor::SIZE],
}

impl ImportDescriptor {
    pub const SIZE: usize = 20;
    const NAME_FIELD: usize = 12;

    pub fn from_view(view: &ByteView<'_>, offset: usize) -> Result<Self, PatchError> {
        let mut raw = [0u8; Self::SIZE];
        raw.copy_from_slice(view.slice(offset, Self::SIZE)?);
        Ok(Self { raw })
    }

    /// A descriptor for a table with a fresh ILT, IAT and name.
    pub fn new(original_first_thunk: u32, name: u32, first_thunk: u32) -> Self {
        let mut raw = [0u8; Self::SIZE];
        raw[0..4].copy_from_slice(&original_first_thunk.to_le_bytes());
        raw[Self::NAME_FIELD..Self::NAME_FIELD + 4].copy_from_slice(&name.to_le_bytes());
        raw[16..20].copy_from_slice(&first_thunk.to_le_bytes());
        Self { raw }
    }

    pub fn is_null(&self) -> bool {
        self.raw.iter().all(|byte| *byte == 0)
    }

    pub fn name_rva(&self) -> u32 {
        u32::from_le_bytes([
            self.raw[Self::NAME_FIELD],
            self.raw[Self::NAME_FIELD + 1],
            self.raw[Self::NAME_FIELD + 2],
            self.raw[Self::NAME_FIELD + 3],
        ])
    }
}
