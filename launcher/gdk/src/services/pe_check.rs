use crate::GdkError;
use std::io::Read;
use std::path::Path;

pub struct PeCheck;

impl PeCheck {
    const HEADER_BYTES: usize = 4096;
    const MACHINE_X64: u16 = 0x8664;

    // `MZ`, an `e_lfanew` inside the header, `PE\0\0` and the x64 machine type
    pub fn is_x64(path: &Path) -> Result<bool, GdkError> {
        let mut header = Vec::with_capacity(Self::HEADER_BYTES);
        std::fs::File::open(path)?.take(Self::HEADER_BYTES as u64).read_to_end(&mut header)?;
        Ok(Self::is_x64_header(&header))
    }

    fn is_x64_header(header: &[u8]) -> bool {
        if header.len() < 0x40 || &header[..2] != b"MZ" {
            return false;
        }
        let offset = u32::from_le_bytes([header[0x3C], header[0x3D], header[0x3E], header[0x3F]]) as usize;
        let Some(end) = offset.checked_add(6) else { return false };
        if end > header.len() || &header[offset..offset + 4] != b"PE\0\0" {
            return false;
        }
        u16::from_le_bytes([header[offset + 4], header[offset + 5]]) == Self::MACHINE_X64
    }
}
