use crate::errors::PatchError;
use crate::services::PeChecksum;
use crate::structs::{ByteView, ImportSection, PeLayout};

const SCN_INITIALIZED_DATA: u32 = 0x0000_0040;
const SCN_MEM_READ: u32 = 0x4000_0000;
const SCN_MEM_WRITE: u32 = 0x8000_0000;

/// Writes a copy of a 64-bit PE file that imports one more symbol.
pub struct ImportPatcher;

impl ImportPatcher {
    /// Appends a section `.amml` that holds a new import directory (every existing descriptor
    /// unchanged, one descriptor for `dll!symbol`, a null descriptor) and points the import data
    /// directory at it. The old import data stays where it is, so the old thunk RVAs stay valid.
    /// A certificate blob that ends the file after the last section is cut off, because the
    /// patched file is no longer signed; the certificate directory entry is always cleared.
    /// The bound-import directory is cleared, the section count, `SizeOfImage` and
    /// `SizeOfInitializedData` are updated and the checksum is recomputed.
    pub fn add_import(input: &[u8], dll: &str, symbol: &str) -> Result<Vec<u8>, PatchError> {
        let layout = PeLayout::parse(input)?;
        let existing = layout.import_descriptors(input)?;
        for descriptor in &existing {
            if layout.import_name(input, descriptor)?.eq_ignore_ascii_case(dll) {
                return Err(PatchError::AlreadyPatched(dll.to_string()));
            }
        }
        let slot = layout.free_section_header(input)?;
        let input = Self::without_certificate(&layout, input)?;

        let section_alignment = layout.section_alignment();
        let file_alignment = layout.file_alignment();
        let virtual_address = Self::align(u64::from(layout.size_of_image()), section_alignment)?;
        let raw_pointer = Self::align(input.len() as u64, file_alignment)?;
        let content = ImportSection::build(&existing, virtual_address, dll, symbol)?;
        let virtual_size = Self::to_u32(content.bytes.len() as u64, "section size")?;
        let raw_size = Self::align(u64::from(virtual_size), file_alignment)?;
        let size_of_image = Self::align(u64::from(virtual_address) + u64::from(virtual_size), section_alignment)?;

        let mut output = input.to_vec();
        output.resize(raw_pointer as usize, 0);
        output.extend_from_slice(&content.bytes);
        output.resize(raw_pointer as usize + raw_size as usize, 0);

        let mut header = [0u8; PeLayout::SECTION_HEADER_SIZE];
        header[..8].copy_from_slice(b".amml\0\0\0");
        header[8..12].copy_from_slice(&virtual_size.to_le_bytes());
        header[12..16].copy_from_slice(&virtual_address.to_le_bytes());
        header[16..20].copy_from_slice(&raw_size.to_le_bytes());
        header[20..24].copy_from_slice(&raw_pointer.to_le_bytes());
        header[36..40].copy_from_slice(&(SCN_INITIALIZED_DATA | SCN_MEM_READ | SCN_MEM_WRITE).to_le_bytes());
        output[slot..slot + header.len()].copy_from_slice(&header);

        let count = u16::try_from(layout.section_count() + 1)
            .map_err(|_| PatchError::TooLarge("too many sections".to_string()))?;
        Self::put_u16(&mut output, layout.number_of_sections_offset(), count);
        Self::put_u32(&mut output, layout.size_of_image_offset(), size_of_image);
        let initialized = Self::u32_at(&output, layout.size_of_initialized_data_offset());
        Self::put_u32(&mut output, layout.size_of_initialized_data_offset(), initialized.saturating_add(raw_size));
        let directory = layout.import_directory_offset();
        Self::put_u32(&mut output, directory, virtual_address);
        Self::put_u32(&mut output, directory + 4, content.directory_size);
        if let Some(security) = layout.security_directory_offset() {
            Self::put_u32(&mut output, security, 0);
            Self::put_u32(&mut output, security + 4, 0);
        }
        if let Some(bound) = layout.bound_import_directory_offset() {
            Self::put_u32(&mut output, bound, 0);
            Self::put_u32(&mut output, bound + 4, 0);
        }
        let checksum = PeChecksum::compute(&output, layout.checksum_offset());
        Self::put_u32(&mut output, layout.checksum_offset(), checksum);
        Ok(output)
    }

    // The input up to the start of a trailing certificate blob, or all of it.
    fn without_certificate<'a>(layout: &PeLayout, input: &'a [u8]) -> Result<&'a [u8], PatchError> {
        let Some(entry) = layout.security_directory_offset() else {
            return Ok(input);
        };
        let start = u64::from(ByteView::new(input).u32_at(entry)?);
        let size = u64::from(ByteView::new(input).u32_at(entry + 4)?);
        let trailing = size != 0 && start >= layout.raw_end() && start + size == input.len() as u64;
        Ok(if trailing { &input[..start as usize] } else { input })
    }

    fn align(value: u64, alignment: u32) -> Result<u32, PatchError> {
        let alignment = u64::from(alignment);
        Self::to_u32(value.div_ceil(alignment) * alignment, "aligned size")
    }

    fn to_u32(value: u64, what: &str) -> Result<u32, PatchError> {
        u32::try_from(value).map_err(|_| PatchError::TooLarge(format!("{what} {value:#x} exceeds 32 bits")))
    }

    fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]])
    }
}
