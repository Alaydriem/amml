pub mod function_table;
pub mod section;

use std::collections::HashMap;

use amml_data::BuildId;

pub use function_table::FunctionTable;
pub use section::Section;

#[derive(Clone, Debug)]
pub struct Image {
    image_base: u64,
    sections: Vec<Section>,
    functions: FunctionTable,
    build: BuildId,
    // IAT slot RVA to the imported function name
    imports: HashMap<u32, String>,
}

impl Image {
    pub fn new(image_base: u64, sections: Vec<Section>, functions: FunctionTable, build: BuildId) -> Self {
        Self { image_base, sections, functions, build, imports: HashMap::new() }
    }

    /// Sets the import names, keyed by the RVA of the IAT slot that holds each imported address.
    pub fn with_imports(mut self, imports: HashMap<u32, String>) -> Self {
        self.imports = imports;
        self
    }

    /// The name of the function imported through the IAT slot at `rva`.
    pub fn import_at(&self, rva: u32) -> Option<&str> {
        self.imports.get(&rva).map(String::as_str)
    }

    pub fn image_base(&self) -> u64 {
        self.image_base
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub fn functions(&self) -> &FunctionTable {
        &self.functions
    }

    pub fn build(&self) -> BuildId {
        self.build
    }

    pub fn section_of(&self, rva: u32) -> Option<&Section> {
        self.sections.iter().find(|s| s.contains(rva))
    }

    /// Bytes `[rva, rva + len)`, or `None` when the range leaves its section.
    pub fn read(&self, rva: u32, len: usize) -> Option<&[u8]> {
        let section = self.section_of(rva)?;
        let start = (rva - section.rva) as usize;
        section.data.get(start..start.checked_add(len)?)
    }

    pub fn read_u32(&self, rva: u32) -> Option<u32> {
        let bytes: [u8; 4] = self.read(rva, 4)?.try_into().ok()?;
        Some(u32::from_le_bytes(bytes))
    }

    /// The RVA the qword at `rva` points to, when the pointer lies in an executable section.
    pub fn code_pointer(&self, rva: u32) -> Option<u32> {
        let target = self.read_u64(rva)?.checked_sub(self.image_base)?;
        let target = u32::try_from(target).ok()?;
        self.section_of(target).is_some_and(|section| section.executable).then_some(target)
    }

    pub fn read_u64(&self, rva: u32) -> Option<u64> {
        let bytes: [u8; 8] = self.read(rva, 8)?.try_into().ok()?;
        Some(u64::from_le_bytes(bytes))
    }
}
