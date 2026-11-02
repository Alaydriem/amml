pub struct PeBytes;

impl PeBytes {
    const PE_OFFSET: usize = 0x80;

    // the smallest header `PeCheck` accepts, padded to `len` bytes
    pub fn with_machine(len: usize, machine: u16) -> Vec<u8> {
        let mut bytes = vec![0u8; len.max(Self::PE_OFFSET + 6)];
        bytes[0] = b'M';
        bytes[1] = b'Z';
        bytes[0x3C..0x40].copy_from_slice(&(Self::PE_OFFSET as u32).to_le_bytes());
        bytes[Self::PE_OFFSET..Self::PE_OFFSET + 4].copy_from_slice(b"PE\0\0");
        bytes[Self::PE_OFFSET + 4..Self::PE_OFFSET + 6].copy_from_slice(&machine.to_le_bytes());
        bytes
    }

    pub fn x64(len: usize) -> Vec<u8> {
        Self::with_machine(len, 0x8664)
    }
}
