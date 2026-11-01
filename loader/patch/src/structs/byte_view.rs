use crate::errors::PatchError;

/// Bounds-checked little-endian reads over a byte slice.
#[derive(Clone, Copy)]
pub struct ByteView<'a> {
    bytes: &'a [u8],
}

impl<'a> ByteView<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn slice(&self, offset: usize, length: usize) -> Result<&'a [u8], PatchError> {
        offset
            .checked_add(length)
            .and_then(|end| self.bytes.get(offset..end))
            .ok_or_else(|| PatchError::Malformed(format!("{length} bytes at {offset:#x} run past the end of the file")))
    }

    pub fn u16_at(&self, offset: usize) -> Result<u16, PatchError> {
        let raw = self.slice(offset, 2)?;
        Ok(u16::from_le_bytes([raw[0], raw[1]]))
    }

    pub fn u32_at(&self, offset: usize) -> Result<u32, PatchError> {
        let raw = self.slice(offset, 4)?;
        Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }

    /// The NUL-terminated string at `offset`, without the terminator.
    pub fn c_str_at(&self, offset: usize) -> Result<&'a [u8], PatchError> {
        let rest = self
            .bytes
            .get(offset..)
            .ok_or_else(|| PatchError::Malformed(format!("string at {offset:#x} starts past the end of the file")))?;
        let length = rest
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(|| PatchError::Malformed(format!("string at {offset:#x} has no terminator")))?;
        Ok(&rest[..length])
    }
}
