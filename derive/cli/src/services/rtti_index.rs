use std::collections::HashMap;

use crate::errors::StepError;
use crate::structs::Image;

const TYPE_DESCRIPTOR_HEADER: u32 = 16;
const COL_SIZE: usize = 24;

/// Maps MSVC RTTI class names to their primary vtables.
#[derive(Clone, Debug)]
pub struct RttiIndex {
    // class name to the RVAs of slot 0 of each primary vtable
    vtables: HashMap<String, Vec<u32>>,
}

impl RttiIndex {
    pub fn new(image: &Image) -> Self {
        let names = Self::collect_descriptors(image);
        let locators = Self::collect_locators(image, &names);
        let vtables = Self::collect_vtables(image, &locators);
        Self { vtables }
    }

    /// The RVA of slot 0 of the only primary vtable of `class`.
    ///
    /// Class names are the raw RTTI names, including the `.?AV` / `.?AU` prefix and the `@@` suffix.
    pub fn vtable(&self, class: &str) -> Result<u32, StepError> {
        match self.vtables.get(class).map(Vec::as_slice) {
            None | Some([]) => Err(StepError::ClassNotFound(class.to_string())),
            Some([only]) => Ok(*only),
            Some(_) => Err(StepError::ClassAmbiguous(class.to_string())),
        }
    }

    /// The function RVA stored in `slot` of the vtable of `class`.
    ///
    /// Class names include the `.?AV` / `.?AU` prefix and the `@@` suffix.
    pub fn slot_target(&self, image: &Image, class: &str, slot: u32) -> Result<u32, StepError> {
        let vtable = self.vtable(class)?;
        Self::targets(image, vtable)
            .nth(slot as usize)
            .ok_or_else(|| StepError::SlotOutOfRange { class: class.to_string(), slot })
    }

    /// The slot of `function` in the vtable of `class`.
    ///
    /// Fails with `AmbiguousSlot` when the function fills more than one slot.
    /// Class names include the `.?AV` / `.?AU` prefix and the `@@` suffix.
    pub fn slot_of(&self, image: &Image, class: &str, function: u32) -> Result<u32, StepError> {
        let vtable = self.vtable(class)?;
        let mut slots = Self::targets(image, vtable)
            .enumerate()
            .filter(|(_, target)| *target == function)
            .map(|(slot, _)| slot as u32);
        match (slots.next(), slots.next()) {
            (None, _) => Err(StepError::NotInVtable { class: class.to_string(), rva: function }),
            (Some(slot), None) => Ok(slot),
            (Some(_), Some(_)) => Err(StepError::AmbiguousSlot { class: class.to_string(), rva: function }),
        }
    }

    /// True when `function` appears in at least one slot of the vtable of `class`.
    ///
    /// Returns false for a missing or ambiguous class.
    /// Class names include the `.?AV` / `.?AU` prefix and the `@@` suffix.
    pub fn in_vtable(&self, image: &Image, class: &str, function: u32) -> bool {
        match self.vtable(class) {
            Ok(vtable) => Self::targets(image, vtable).any(|target| target == function),
            Err(_) => false,
        }
    }

    // yields slot targets from `vtable` until the first slot outside executable code
    fn targets(image: &Image, vtable: u32) -> impl Iterator<Item = u32> + '_ {
        (0u32..).map_while(move |slot| {
            let rva = vtable.checked_add(slot.checked_mul(8)?)?;
            Self::code_rva(image, image.read_u64(rva)?)
        })
    }

    // the RVA of an absolute address when it lands in an executable section
    fn code_rva(image: &Image, address: u64) -> Option<u32> {
        let rva = u32::try_from(address.checked_sub(image.image_base())?).ok()?;
        image.section_of(rva).filter(|s| s.executable).map(|_| rva)
    }

    // type descriptor RVA to class name, from every `.?AV` / `.?AU` name outside code
    fn collect_descriptors(image: &Image) -> HashMap<u32, String> {
        let mut descriptors = HashMap::new();
        for section in image.sections().iter().filter(|s| !s.executable) {
            for at in memchr::memmem::find_iter(&section.data, b".?A") {
                if at > 0 && section.data[at - 1] != 0 {
                    continue;
                }
                let Some(kind) = section.data.get(at + 3) else { continue };
                if *kind != b'V' && *kind != b'U' {
                    continue;
                }
                let tail = &section.data[at..];
                let len = memchr::memchr(0, tail).unwrap_or(tail.len());
                let Ok(name) = std::str::from_utf8(&tail[..len]) else { continue };
                let Some(name_rva) = section.rva.checked_add(at as u32) else { continue };
                let Some(descriptor) = name_rva.checked_sub(TYPE_DESCRIPTOR_HEADER) else { continue };
                descriptors.insert(descriptor, name.to_string());
            }
        }
        descriptors
    }

    // one 4-byte-aligned pass over .rdata; locator RVA to class name
    fn collect_locators(image: &Image, descriptors: &HashMap<u32, String>) -> HashMap<u32, String> {
        let mut locators = HashMap::new();
        if descriptors.is_empty() {
            return locators;
        }
        for section in image.sections().iter().filter(|s| s.name == ".rdata") {
            let data = &section.data;
            let word = |at: usize| u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
            let mut at = (4 - (section.rva % 4) as usize) % 4;
            while at + COL_SIZE <= data.len() {
                if word(at) == 1 && word(at + 4) == 0 {
                    let own = section.rva.wrapping_add(at as u32);
                    if word(at + 20) == own
                        && let Some(name) = descriptors.get(&word(at + 12))
                    {
                        locators.insert(own, name.clone());
                    }
                }
                at += 4;
            }
        }
        locators
    }

    // one 8-byte-aligned pass over .rdata for pointers to a known locator
    fn collect_vtables(image: &Image, locators: &HashMap<u32, String>) -> HashMap<String, Vec<u32>> {
        let mut vtables: HashMap<String, Vec<u32>> = HashMap::new();
        if locators.is_empty() {
            return vtables;
        }
        let base = image.image_base();
        for section in image.sections().iter().filter(|s| s.name == ".rdata") {
            let data = &section.data;
            let mut at = (8 - (section.rva % 8) as usize) % 8;
            while at + 8 <= data.len() {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[at..at + 8]);
                let value = u64::from_le_bytes(bytes);
                if let Some(col) = value.checked_sub(base).and_then(|rva| u32::try_from(rva).ok())
                    && let Some(name) = locators.get(&col)
                {
                    let vtable = section.rva.wrapping_add(at as u32).wrapping_add(8);
                    let has_code_slot = image.read_u64(vtable).and_then(|slot| Self::code_rva(image, slot)).is_some();
                    if !has_code_slot {
                        at += 8;
                        continue;
                    }
                    vtables.entry(name.clone()).or_default().push(vtable);
                }
                at += 8;
            }
        }
        vtables
    }
}
