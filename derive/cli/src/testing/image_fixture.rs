use std::collections::HashMap;

use amml_data::BuildId;
use xxhash_rust::xxh3::xxh3_64;

use crate::structs::{FunctionTable, Image, Section};

const TEXT_RVA: u32 = 0x1000;
const RDATA_RVA: u32 = 0x40_0000;
const DATA_RVA: u32 = 0x80_0000;

pub struct ImageFixture {
    image_base: u64,
    text: Vec<u8>,
    rdata: Vec<u8>,
    data: Vec<u8>,
    functions: Vec<(u32, u32)>,
    fragments: Vec<(u32, u32, u32)>,
    imports: HashMap<u32, String>,
}

impl ImageFixture {
    pub fn new(image_base: u64) -> Self {
        Self {
            image_base,
            text: Vec::new(),
            rdata: Vec::new(),
            data: Vec::new(),
            functions: Vec::new(),
            fragments: Vec::new(),
            imports: HashMap::new(),
        }
    }

    pub fn put_text(&mut self, bytes: &[u8]) -> u32 {
        TEXT_RVA + Self::append(&mut self.text, bytes)
    }

    pub fn put_rdata(&mut self, bytes: &[u8]) -> u32 {
        RDATA_RVA + Self::append(&mut self.rdata, bytes)
    }

    pub fn put_data(&mut self, bytes: &[u8]) -> u32 {
        DATA_RVA + Self::append(&mut self.data, bytes)
    }

    /// An IAT slot for the import `name`; returns the RVA of the slot.
    pub fn put_import(&mut self, name: &str) -> u32 {
        let slot = self.put_data(&[0; 8]);
        self.imports.insert(slot, name.to_string());
        slot
    }

    pub fn put_function(&mut self, start: u32, end: u32) {
        self.functions.push((start, end));
    }

    /// A chained fragment of the function whose primary starts at `primary`.
    pub fn put_fragment(&mut self, start: u32, end: u32, primary: u32) {
        self.fragments.push((start, end, primary));
    }

    pub fn image(&self) -> Image {
        let sections = vec![
            Section::new(".text".into(), TEXT_RVA, self.text.clone(), true),
            Section::new(".rdata".into(), RDATA_RVA, self.rdata.clone(), false),
            Section::new(".data".into(), DATA_RVA, self.data.clone(), false),
        ];
        let build = BuildId::new(0x5eed, xxh3_64(&self.text));
        Image::new(
            self.image_base,
            sections,
            FunctionTable::with_fragments(self.functions.clone(), self.fragments.clone()),
            build,
        )
        .with_imports(self.imports.clone())
    }

    /// Writes a type descriptor, a complete object locator and a vtable; returns the vtable RVA.
    pub fn put_class(&mut self, name: &str, slots: &[u32]) -> u32 {
        self.put_class_with_locator(name, slots, 1, 0, 0)
    }

    /// Like `put_class`, with the locator signature, vtable offset and a skew added to its self RVA.
    pub fn put_class_with_locator(
        &mut self,
        name: &str,
        slots: &[u32],
        signature: u32,
        offset: u32,
        self_skew: u32,
    ) -> u32 {
        let mut descriptor = vec![0u8; 16];
        descriptor.extend_from_slice(name.as_bytes());
        descriptor.push(0);
        let descriptor_rva = self.put_rdata(&descriptor);
        // the locator's own RVA is the next 16-byte-aligned rdata offset
        let locator_rva = RDATA_RVA + self.rdata.len().next_multiple_of(16) as u32;
        let mut locator = Vec::with_capacity(24);
        for word in [signature, offset, 0, descriptor_rva, 0, locator_rva.wrapping_add(self_skew)] {
            locator.extend_from_slice(&u32::to_le_bytes(word));
        }
        self.put_rdata(&locator);
        let mut vtable = (self.image_base + u64::from(locator_rva)).to_le_bytes().to_vec();
        for slot in slots {
            vtable.extend_from_slice(&(self.image_base + u64::from(*slot)).to_le_bytes());
        }
        self.put_rdata(&vtable) + 8
    }

    // pads to 16 bytes, appends, and returns the offset of the appended bytes
    fn append(section: &mut Vec<u8>, bytes: &[u8]) -> u32 {
        section.resize(section.len().next_multiple_of(16), 0);
        let offset = section.len() as u32;
        section.extend_from_slice(bytes);
        offset
    }
}
