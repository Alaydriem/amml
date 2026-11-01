use std::ops::Range;

/// Where the game image lies in memory, as RVAs: its size and the executable sections.
///
/// A LeviLamina address is accepted only when it falls inside this layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImageLayout {
    size_of_image: u64,
    executable: Vec<Range<u64>>,
}

impl ImageLayout {
    const SECTION_HEADER_SIZE: usize = 40;
    const EXECUTE_FLAG: u32 = 0x2000_0000;
    const PE_SIGNATURE: u32 = 0x0000_4550;
    const OPTIONAL_MAGIC_64: u16 = 0x20b;

    /// A layout of `size_of_image` bytes with the given executable RVA ranges.
    pub fn new(size_of_image: u64, executable: Vec<Range<u64>>) -> Self {
        Self { size_of_image, executable }
    }

    /// A layout that contains nothing, so no address passes.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Reads the layout from the headers of a 64-bit PE image: `headers` starts at the DOS header and
    /// must hold the section table. `None` when the bytes are not such a PE or end early.
    pub fn from_headers(headers: &[u8]) -> Option<Self> {
        let pe = Self::u32_at(headers, 0x3c)? as usize;
        if Self::u32_at(headers, pe)? != Self::PE_SIGNATURE {
            return None;
        }
        let sections = usize::from(Self::u16_at(headers, pe + 6)?);
        let optional_size = usize::from(Self::u16_at(headers, pe + 20)?);
        let optional = pe + 24;
        if Self::u16_at(headers, optional)? != Self::OPTIONAL_MAGIC_64 {
            return None;
        }
        let size_of_image = u64::from(Self::u32_at(headers, optional + 56)?);
        let table = optional.checked_add(optional_size)?;
        let mut executable = Vec::new();
        for index in 0..sections {
            let header = table.checked_add(index.checked_mul(Self::SECTION_HEADER_SIZE)?)?;
            let virtual_size = u64::from(Self::u32_at(headers, header + 8)?);
            let raw_size = u64::from(Self::u32_at(headers, header + 16)?);
            let start = u64::from(Self::u32_at(headers, header + 12)?);
            let flags = Self::u32_at(headers, header + 36)?;
            if flags & Self::EXECUTE_FLAG != 0 {
                let length = if virtual_size == 0 { raw_size } else { virtual_size };
                executable.push(start..start.saturating_add(length));
            }
        }
        Some(Self { size_of_image, executable })
    }

    pub fn size_of_image(&self) -> u64 {
        self.size_of_image
    }

    /// Whether `rva` is inside the image.
    pub fn contains(&self, rva: u64) -> bool {
        rva < self.size_of_image
    }

    /// Whether `rva` is inside an executable section.
    pub fn is_executable(&self, rva: u64) -> bool {
        self.contains(rva) && self.executable.iter().any(|range| range.contains(&rva))
    }

    fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
        let slice = bytes.get(at..at.checked_add(2)?)?;
        Some(u16::from_le_bytes(slice.try_into().ok()?))
    }

    fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
        let slice = bytes.get(at..at.checked_add(4)?)?;
        Some(u32::from_le_bytes(slice.try_into().ok()?))
    }
}
