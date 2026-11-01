use std::collections::BTreeSet;

use memchr::{memchr_iter, memmem};

use crate::errors::StepError;
use crate::structs::Image;

const MIN_STRING: usize = 4;
const MAX_STRING: usize = 512;

/// Every `lea r64, [rip+disp32]` of an image, indexed by the address it loads.
///
/// Built once per image and shared by all `string_xref` steps of a run.
#[derive(Clone, Debug)]
pub struct StringXref {
    // (loaded rva, rva of the lea), sorted
    loads: Vec<(u32, u32)>,
    // (rva of the lea, loaded rva), sorted
    by_lea: Vec<(u32, u32)>,
}

impl StringXref {
    pub fn new(image: &Image) -> Self {
        let mut loads = Vec::new();
        for section in image.sections().iter().filter(|s| s.executable) {
            let data = &section.data;
            // REX.W (48) or REX.WR (4C), opcode 8D, ModRM with mod 00 and rm 101
            for opcode_at in memchr_iter(0x8D, data) {
                let Some(start) = opcode_at.checked_sub(1) else { continue };
                if !matches!(data[start], 0x48 | 0x4C) || start + 7 > data.len() || data[start + 2] & 0xC7 != 0x05 {
                    continue;
                }
                let disp = i32::from_le_bytes([data[start + 3], data[start + 4], data[start + 5], data[start + 6]]);
                let Some(lea) = u32::try_from(start).ok().and_then(|offset| section.rva.checked_add(offset)) else {
                    continue;
                };
                let Some(target) = lea.checked_add(7).and_then(|next| next.checked_add_signed(disp)) else {
                    continue;
                };
                loads.push((target, lea));
            }
        }
        loads.sort_unstable();
        let mut by_lea: Vec<(u32, u32)> = loads.iter().map(|&(loaded, lea)| (lea, loaded)).collect();
        by_lea.sort_unstable();
        Self { loads, by_lea }
    }

    /// Start RVAs of the functions holding a `lea r64, [rip+disp32]` that loads `text` followed
    /// by a NUL byte, ascending and without duplicates. A `lea` outside every function is ignored.
    pub fn functions_using(&self, image: &Image, text: &str) -> Result<Vec<u32>, StepError> {
        let targets = Self::string_rvas(image, text);
        if targets.is_empty() {
            return Err(StepError::StringNotFound(text.to_owned()));
        }
        let mut starts = BTreeSet::new();
        for target in targets {
            let first = self.loads.partition_point(|&(loaded, _)| loaded < target);
            for &(loaded, lea) in &self.loads[first..] {
                if loaded != target {
                    break;
                }
                if let Some((primary, _)) = image.functions().containing(lea) {
                    starts.insert(primary);
                }
            }
        }
        Ok(starts.into_iter().collect())
    }

    /// The strings that a `lea r64, [rip+disp32]` inside `function` loads, in address order and without
    /// duplicates. A string lies in a non-executable section, ends with a NUL byte, has at least 4 bytes and holds
    /// only printable ASCII. The ranges of `function` come from the function table, chained fragments included.
    pub fn strings_used_by(&self, image: &Image, function: u32) -> Vec<String> {
        let mut found = BTreeSet::new();
        for (start, end) in image.functions().ranges_of(function) {
            let first = self.by_lea.partition_point(|&(lea, _)| lea < start);
            for &(lea, loaded) in &self.by_lea[first..] {
                if lea >= end {
                    break;
                }
                found.insert(loaded);
            }
        }
        found.into_iter().filter_map(|rva| Self::string_at(image, rva)).fold(Vec::new(), |mut strings, text| {
            if !strings.contains(&text) {
                strings.push(text);
            }
            strings
        })
    }

    // the printable NUL-terminated string of at least MIN_STRING bytes at `rva` in a non-executable section
    fn string_at(image: &Image, rva: u32) -> Option<String> {
        let section = image.section_of(rva).filter(|section| !section.executable)?;
        let start = usize::try_from(rva - section.rva).ok()?;
        let tail = section.data.get(start..)?;
        let end = tail.iter().take(MAX_STRING).position(|&byte| byte == 0)?;
        let bytes = &tail[..end];
        (bytes.len() >= MIN_STRING && bytes.iter().all(|byte| (0x20..=0x7E).contains(byte)))
            .then(|| String::from_utf8_lossy(bytes).into_owned())
    }

    // RVAs where `text` begins a NUL-terminated string in a non-executable section
    fn string_rvas(image: &Image, text: &str) -> BTreeSet<u32> {
        let mut needle = text.as_bytes().to_vec();
        needle.push(0);
        let finder = memmem::Finder::new(&needle);
        let mut found = BTreeSet::new();
        for section in image.sections().iter().filter(|s| !s.executable) {
            for hit in finder.find_iter(&section.data) {
                if hit != 0 && section.data[hit - 1] != 0 {
                    continue;
                }
                let Ok(offset) = u32::try_from(hit) else { continue };
                if let Some(rva) = section.rva.checked_add(offset) {
                    found.insert(rva);
                }
            }
        }
        found
    }
}
