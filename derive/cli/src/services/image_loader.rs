use std::collections::HashMap;
use std::path::Path;

use amml_data::BuildIdReader;
use pelite::pe64::imports::Import;
use pelite::pe64::{Pe, PeFile};

use crate::errors::DeriveError;
use crate::structs::{FunctionTable, Image, Section};

const UNW_FLAG_CHAININFO: u8 = 0x04;
const SCN_MEM_EXECUTE: u32 = 0x2000_0000;

pub struct ImageLoader;

impl ImageLoader {
    pub fn from_path(path: &Path) -> Result<Image, DeriveError> {
        let bytes = std::fs::read(path).map_err(|source| DeriveError::Io { path: path.to_path_buf(), source })?;
        Self::from_pe_bytes(&bytes)
    }

    pub fn from_pe_bytes(bytes: &[u8]) -> Result<Image, DeriveError> {
        let not_pe = |e: pelite::Error| DeriveError::NotPe(e.to_string());
        let file = PeFile::from_bytes(bytes).map_err(not_pe)?;

        let build = BuildIdReader::from_bytes(bytes).map_err(|error| match error {
            amml_data::DataError::NotPe { message, .. } => DeriveError::NotPe(message),
            other => DeriveError::NotPe(other.to_string()),
        })?;

        let mut sections = Vec::new();
        for header in file.section_headers() {
            let raw = file.get_section_bytes(header).map_err(not_pe)?;
            let executable = header.Characteristics & SCN_MEM_EXECUTE != 0;
            let mut data = raw.to_vec();
            if (header.VirtualSize as usize) > data.len() {
                data.resize(header.VirtualSize as usize, 0);
            }
            let name_bytes = header.Name.split(|&b| b == 0).next().unwrap_or(&[]);
            let name = String::from_utf8_lossy(name_bytes).into_owned();
            sections.push(Section::new(name, header.VirtualAddress, data, executable));
        }

        let functions = match file.exception() {
            Ok(table) => Self::function_table(&file, table.functions()),
            Err(pelite::Error::Null) => FunctionTable::new(Vec::new()),
            Err(e) => return Err(not_pe(e)),
        };

        let image = Image::new(file.optional_header().ImageBase, sections, functions, build);
        Ok(image.with_imports(Self::import_names(&file)))
    }

    // A file without an import table, or a table that does not read, has no import names.
    fn import_names(file: &PeFile<'_>) -> HashMap<u32, String> {
        let mut names = HashMap::new();
        let Ok(imports) = file.imports() else { return names };
        for descriptor in imports {
            let (Ok(table), Ok(slots)) = (descriptor.int(), descriptor.iat()) else { continue };
            let first = descriptor.image().FirstThunk;
            for (index, (import, _)) in table.zip(slots).enumerate() {
                let Ok(Import::ByName { name, .. }) = import else { continue };
                let Ok(name) = name.to_str() else { continue };
                let Some(slot) = u32::try_from(index).ok().and_then(|i| first.checked_add(i * 8)) else { continue };
                names.insert(slot, name.to_string());
            }
        }
        names
    }

    // chained entries become fragments of their primary function
    fn function_table<'a>(
        file: &PeFile<'a>,
        functions: impl Iterator<Item = pelite::pe64::exception::Function<'a, PeFile<'a>>>,
    ) -> FunctionTable {
        let mut entries = Vec::new();
        let mut parents = HashMap::new();
        for f in functions {
            let (begin, end) = (f.image().BeginAddress, f.image().EndAddress);
            entries.push((begin, end));
            if let Some(parent) = Self::chain_parent(file, &f) {
                parents.insert(begin, parent);
            }
        }
        let mut primaries = Vec::new();
        let mut fragments = Vec::new();
        for (begin, end) in entries {
            if !parents.contains_key(&begin) {
                primaries.push((begin, end));
                continue;
            }
            let mut primary = begin;
            // bounded so a malformed cycle cannot loop
            for _ in 0..16 {
                match parents.get(&primary) {
                    Some(&parent) => primary = parent,
                    None => break,
                }
            }
            if parents.contains_key(&primary) {
                primaries.push((begin, end));
            } else {
                fragments.push((begin, end, primary));
            }
        }
        FunctionTable::with_fragments(primaries, fragments)
    }

    // begin address of the parent entry when the unwind info is chained
    fn chain_parent<'a>(file: &PeFile<'a>, f: &pelite::pe64::exception::Function<'a, PeFile<'a>>) -> Option<u32> {
        let info = f.unwind_info().ok()?;
        if info.flags() & UNW_FLAG_CHAININFO == 0 {
            return None;
        }
        let codes = (info.unwind_codes().len() as u32).next_multiple_of(2);
        let at = f.image().UnwindData.checked_add(4 + 2 * codes)?;
        let bytes = file.derva_slice::<u8>(at, 4).ok()?;
        Some(u32::from_le_bytes(bytes.try_into().ok()?))
    }
}
