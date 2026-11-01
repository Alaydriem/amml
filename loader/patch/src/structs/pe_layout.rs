use crate::errors::PatchError;
use crate::structs::{ByteView, ImportDescriptor, SectionInfo};

const PE_SIGNATURE: u32 = 0x0000_4550;
const MAGIC_PE32_PLUS: u16 = 0x20b;
const SECTION_HEADER_SIZE: usize = 40;
const IMPORT_DIRECTORY: usize = 1;
const SECURITY_DIRECTORY: usize = 4;
const BOUND_IMPORT_DIRECTORY: usize = 11;
const DIRECTORY_TABLE: usize = 112;

/// Where the fields the patcher touches sit in a 64-bit PE file, and the section table.
#[derive(Clone, Debug)]
pub struct PeLayout {
    file_header: usize,
    optional_header: usize,
    section_count: usize,
    section_table: usize,
    sections: Vec<SectionInfo>,
    section_alignment: u32,
    file_alignment: u32,
    size_of_image: u32,
    size_of_headers: u32,
    directory_count: u32,
}

impl PeLayout {
    pub const SECTION_HEADER_SIZE: usize = SECTION_HEADER_SIZE;

    pub fn parse(bytes: &[u8]) -> Result<Self, PatchError> {
        let view = ByteView::new(bytes);
        if view.slice(0, 2)? != b"MZ" {
            return Err(PatchError::Malformed("missing the MZ signature".to_string()));
        }
        let pe = view.u32_at(0x3c)? as usize;
        if view.u32_at(pe)? != PE_SIGNATURE {
            return Err(PatchError::Malformed("missing the PE signature".to_string()));
        }
        let file_header = pe + 4;
        let optional_header = file_header + 20;
        if view.u16_at(optional_header)? != MAGIC_PE32_PLUS {
            return Err(PatchError::Malformed("only 64-bit images are supported".to_string()));
        }
        let section_count = usize::from(view.u16_at(file_header + 2)?);
        let optional_size = usize::from(view.u16_at(file_header + 16)?);
        let section_table = optional_header + optional_size;
        let mut sections = Vec::with_capacity(section_count);
        for index in 0..section_count {
            let header = section_table + index * SECTION_HEADER_SIZE;
            sections.push(SectionInfo {
                virtual_size: view.u32_at(header + 8)?,
                virtual_address: view.u32_at(header + 12)?,
                raw_size: view.u32_at(header + 16)?,
                raw_pointer: view.u32_at(header + 20)?,
            });
        }
        let directory_count = view.u32_at(optional_header + 108)?;
        let table_end = DIRECTORY_TABLE as u64 + 8 * u64::from(directory_count);
        if table_end > optional_size as u64 {
            return Err(PatchError::Malformed("the data directory runs past the optional header".to_string()));
        }
        // the checksum field and the whole directory table must be inside the file
        view.slice(optional_header + 64, 4)?;
        view.slice(optional_header + DIRECTORY_TABLE, table_end as usize - DIRECTORY_TABLE)?;
        let layout = Self {
            file_header,
            optional_header,
            section_count,
            section_table,
            sections,
            section_alignment: view.u32_at(optional_header + 32)?,
            file_alignment: view.u32_at(optional_header + 36)?,
            size_of_image: view.u32_at(optional_header + 56)?,
            size_of_headers: view.u32_at(optional_header + 60)?,
            directory_count,
        };
        if !layout.section_alignment.is_power_of_two() || !layout.file_alignment.is_power_of_two() {
            return Err(PatchError::Malformed("the alignments are not powers of two".to_string()));
        }
        if layout.directory_count as usize <= IMPORT_DIRECTORY {
            return Err(PatchError::Malformed("the data directory has no import entry".to_string()));
        }
        Ok(layout)
    }

    pub fn section_alignment(&self) -> u32 {
        self.section_alignment
    }

    pub fn file_alignment(&self) -> u32 {
        self.file_alignment
    }

    pub fn size_of_image(&self) -> u32 {
        self.size_of_image
    }

    pub fn section_count(&self) -> usize {
        self.section_count
    }

    pub fn number_of_sections_offset(&self) -> usize {
        self.file_header + 2
    }

    pub fn size_of_image_offset(&self) -> usize {
        self.optional_header + 56
    }

    pub fn size_of_initialized_data_offset(&self) -> usize {
        self.optional_header + 8
    }

    pub fn checksum_offset(&self) -> usize {
        self.optional_header + 64
    }

    fn directory_offset(&self, index: usize) -> usize {
        self.optional_header + 112 + index * 8
    }

    /// The offset of the certificate-table directory entry, when the image has one. Unlike the
    /// other entries, its address field is a file offset.
    pub fn security_directory_offset(&self) -> Option<usize> {
        (self.directory_count as usize > SECURITY_DIRECTORY).then(|| self.directory_offset(SECURITY_DIRECTORY))
    }

    /// The end of the headers and of the raw data of every section.
    pub fn raw_end(&self) -> u64 {
        self.sections
            .iter()
            .map(|section| u64::from(section.raw_pointer) + u64::from(section.raw_size))
            .max()
            .unwrap_or(0)
            .max(u64::from(self.size_of_headers))
    }

    pub fn import_directory_offset(&self) -> usize {
        self.directory_offset(IMPORT_DIRECTORY)
    }

    /// The offset of the bound-import directory entry, when the image has one.
    pub fn bound_import_directory_offset(&self) -> Option<usize> {
        (self.directory_count as usize > BOUND_IMPORT_DIRECTORY).then(|| self.directory_offset(BOUND_IMPORT_DIRECTORY))
    }

    /// The file offset of the first unused section header, when there is room for one.
    pub fn free_section_header(&self, bytes: &[u8]) -> Result<usize, PatchError> {
        let slot = self.section_table + self.section_count * SECTION_HEADER_SIZE;
        let first_raw = self
            .sections
            .iter()
            .filter(|section| section.raw_size != 0)
            .map(|section| section.raw_pointer)
            .min()
            .unwrap_or(self.size_of_headers);
        let limit = self.size_of_headers.min(first_raw) as usize;
        let free = ByteView::new(bytes).slice(slot, SECTION_HEADER_SIZE).ok();
        match free {
            Some(slice) if slot + SECTION_HEADER_SIZE <= limit && slice.iter().all(|byte| *byte == 0) => Ok(slot),
            _ => Err(PatchError::NoHeaderRoom),
        }
    }

    /// The file offset behind an RVA, for data that is stored in the file.
    pub fn rva_to_offset(&self, rva: u32) -> Result<usize, PatchError> {
        if rva < self.size_of_headers {
            return Ok(rva as usize);
        }
        self.sections
            .iter()
            .find_map(|section| {
                let delta = rva.checked_sub(section.virtual_address)?;
                (delta < section.raw_size).then(|| section.raw_pointer as usize + delta as usize)
            })
            .ok_or_else(|| PatchError::Malformed(format!("rva {rva:#x} is not backed by file data")))
    }

    /// Every non-null import descriptor, in file order.
    pub fn import_descriptors(&self, bytes: &[u8]) -> Result<Vec<ImportDescriptor>, PatchError> {
        let view = ByteView::new(bytes);
        let rva = view.u32_at(self.import_directory_offset())?;
        if rva == 0 {
            return Ok(Vec::new());
        }
        let start = self.rva_to_offset(rva)?;
        let mut found = Vec::new();
        loop {
            let descriptor = ImportDescriptor::from_view(&view, start + found.len() * ImportDescriptor::SIZE)?;
            if descriptor.is_null() {
                return Ok(found);
            }
            found.push(descriptor);
        }
    }

    /// The DLL name an import descriptor points at.
    pub fn import_name(&self, bytes: &[u8], descriptor: &ImportDescriptor) -> Result<String, PatchError> {
        let offset = self.rva_to_offset(descriptor.name_rva())?;
        let name = ByteView::new(bytes).c_str_at(offset)?;
        Ok(String::from_utf8_lossy(name).into_owned())
    }
}
