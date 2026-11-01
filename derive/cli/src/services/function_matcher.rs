use std::cell::OnceCell;
use std::collections::HashMap;

use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction, MemorySize, OpKind};

use crate::errors::StepError;
use crate::structs::{Fingerprint, FunctionIndex, Image, MatchCandidate};

// Immediates below this are mostly offsets and sizes that move between builds
const LARGE_CONSTANT: u64 = 0x10000;
const MIN_STRING_LEN: usize = 3;
const MAX_STRING_LEN: usize = 256;

/// Finds the functions of one build that look like a function of another build.
///
/// The match uses only what survives a rebuild: the strings a function references, the imports it calls and its
/// large constants. The result is advisory: it never proves that two functions are the same.
pub struct FunctionMatcher<'a> {
    old: &'a Image,
    new: &'a Image,
    index: OnceCell<FunctionIndex>,
}

impl<'a> FunctionMatcher<'a> {
    pub fn new(old: &'a Image, new: &'a Image) -> Self {
        Self { old, new, index: OnceCell::new() }
    }

    /// The start and the fingerprint of the old function that holds `old_rva`.
    pub fn old_fingerprint(&self, old_rva: u32) -> Result<(u32, Fingerprint), StepError> {
        let (start, _) = self.old.functions().containing(old_rva).ok_or(StepError::NotInListedFunction(old_rva))?;
        Ok((start, Self::fingerprint(self.old, &self.old.functions().ranges_of(start))))
    }

    /// The `top` functions of the new build that share the most strings, imports and constants with the old
    /// function that holds `old_rva`. Sorted by score, then by RVA.
    pub fn candidates(&self, old_rva: u32, top: usize) -> Result<Vec<MatchCandidate>, StepError> {
        let (start, old) = self.old_fingerprint(old_rva)?;
        if old.anchors() == 0 {
            return Err(StepError::NothingToMatch(start));
        }
        if self.index.get().is_none() {
            let built = Self::build_index(self.new)?;
            // a set that fails means another call filled it first, which cannot happen on one thread
            let _ = self.index.set(built);
        }
        let Some(index) = self.index.get() else { return Err(StepError::NothingToMatch(start)) };

        // shared strings, imports and constants per new function
        let mut shared: HashMap<u32, [usize; 3]> = HashMap::new();
        for text in &old.strings {
            Self::count(index.strings.get(text), 0, &mut shared);
        }
        for name in &old.imports {
            Self::count(index.imports.get(name), 1, &mut shared);
        }
        for value in &old.constants {
            Self::count(index.constants.get(value), 2, &mut shared);
        }

        let mut scored: Vec<(MatchCandidate, f64)> = shared
            .into_iter()
            .filter_map(|(rva, [strings, imports, constants])| {
                let &(anchors, size) = index.functions.get(&rva)?;
                let inter = strings + imports + constants;
                let union = old.anchors() + anchors - inter;
                let (small, large) = (old.size.min(size), old.size.max(size));
                let size_similarity = if large == 0 { 1.0 } else { f64::from(small) / f64::from(large) };
                let candidate = MatchCandidate {
                    rva,
                    score: inter as f64 / union as f64,
                    shared_strings: strings,
                    shared_constants: constants,
                    shared_imports: imports,
                    size,
                };
                Some((candidate, size_similarity))
            })
            .collect();
        scored.sort_by(|(a, a_size), (b, b_size)| {
            b.score.total_cmp(&a.score).then(b_size.total_cmp(a_size)).then(a.rva.cmp(&b.rva))
        });
        Ok(scored.into_iter().take(top).map(|(candidate, _)| candidate).collect())
    }

    fn count(list: Option<&Vec<u32>>, slot: usize, shared: &mut HashMap<u32, [usize; 3]>) {
        for &rva in list.into_iter().flatten() {
            shared.entry(rva).or_default()[slot] += 1;
        }
    }

    fn build_index(image: &Image) -> Result<FunctionIndex, StepError> {
        let starts = image.functions().grouped();
        let threads = std::thread::available_parallelism().map_or(1, usize::from);
        let chunk = starts.len().div_ceil(threads).max(1);
        let prints: Vec<(u32, Fingerprint)> = std::thread::scope(|scope| {
            let workers: Vec<_> = starts
                .chunks(chunk)
                .map(|part| {
                    scope.spawn(move || {
                        part.iter()
                            .map(|(start, ranges)| (*start, Self::fingerprint(image, ranges)))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            let mut all = Vec::new();
            for worker in workers {
                match worker.join() {
                    Ok(part) => all.extend(part),
                    Err(_) => {
                        return Err(StepError::Undecodable("a worker failed while indexing the functions".into()));
                    }
                }
            }
            Ok(all)
        })?;
        let mut index = FunctionIndex::default();
        for (start, print) in prints {
            index.functions.insert(start, (print.anchors(), print.size));
            for text in print.strings {
                index.strings.entry(text).or_default().push(start);
            }
            for name in print.imports {
                index.imports.entry(name).or_default().push(start);
            }
            for value in print.constants {
                index.constants.entry(value).or_default().push(start);
            }
        }
        Ok(index)
    }

    // Decodes every range of the function. A byte that does not decode ends its range.
    fn fingerprint(image: &Image, ranges: &[(u32, u32)]) -> Fingerprint {
        let mut print = Fingerprint::default();
        for &(from, to) in ranges {
            print.size += to - from;
            let Some(bytes) = image.read(from, (to - from) as usize) else { continue };
            let mut decoder = Decoder::with_ip(64, bytes, u64::from(from), DecoderOptions::NONE);
            let mut instruction = Instruction::default();
            while decoder.can_decode() {
                decoder.decode_out(&mut instruction);
                if instruction.is_invalid() {
                    break;
                }
                Self::collect(image, &instruction, &mut print);
            }
        }
        print
    }

    fn collect(image: &Image, instruction: &Instruction, print: &mut Fingerprint) {
        if instruction.flow_control() == FlowControl::Call {
            print.calls += 1;
        }
        if instruction.is_ip_rel_memory_operand() {
            Self::collect_operand(image, instruction, print);
        }
        for operand in 0..instruction.op_count() {
            if let Some(value) = Self::large_immediate(instruction, operand) {
                print.constants.insert(value);
            }
        }
    }

    fn collect_operand(image: &Image, instruction: &Instruction, print: &mut Fingerprint) {
        let Ok(target) = u32::try_from(instruction.ip_rel_memory_address()) else { return };
        if let Some(name) = image.import_at(target) {
            print.imports.insert(name.to_string());
            return;
        }
        let Some(section) = image.section_of(target).filter(|section| !section.executable) else { return };
        let width = match instruction.memory_size() {
            MemorySize::Float32 => 4,
            MemorySize::Float64 => 8,
            _ => 0,
        };
        if width != 0 {
            let Some(bytes) = image.read(target, width) else { return };
            let mut word = [0u8; 8];
            word[..width].copy_from_slice(bytes);
            let value = u64::from_le_bytes(word);
            if value >= LARGE_CONSTANT {
                print.constants.insert(value);
            }
            return;
        }
        let offset = (target - section.rva) as usize;
        if let Some(text) = Self::string_at(&section.data[offset..]) {
            print.strings.insert(text);
        }
    }

    // A printable ASCII run that ends in a NUL byte
    fn string_at(data: &[u8]) -> Option<String> {
        let end = data.iter().take(MAX_STRING_LEN + 1).position(|&byte| byte == 0)?;
        let text = &data[..end];
        let printable = text.iter().all(|&byte| matches!(byte, b' '..=b'~' | b'\t' | b'\n' | b'\r'));
        (end >= MIN_STRING_LEN && printable).then(|| String::from_utf8_lossy(text).into_owned())
    }

    // The immediate as a signed value: negative ones and ones of 0x10000 or more are constants
    fn large_immediate(instruction: &Instruction, operand: u32) -> Option<u64> {
        if !matches!(
            instruction.op_kind(operand),
            OpKind::Immediate8
                | OpKind::Immediate8_2nd
                | OpKind::Immediate16
                | OpKind::Immediate32
                | OpKind::Immediate64
                | OpKind::Immediate8to16
                | OpKind::Immediate8to32
                | OpKind::Immediate8to64
                | OpKind::Immediate32to64
        ) {
            return None;
        }
        let immediate = instruction.immediate(operand);
        let signed = match instruction.op_kind(operand) {
            OpKind::Immediate8 | OpKind::Immediate8_2nd => i64::from(immediate as u8 as i8),
            OpKind::Immediate16 => i64::from(immediate as u16 as i16),
            OpKind::Immediate32 => i64::from(immediate as u32 as i32),
            OpKind::Immediate8to16 | OpKind::Immediate8to32 | OpKind::Immediate8to64 => {
                i64::from(immediate as u8 as i8)
            }
            OpKind::Immediate32to64 => i64::from(immediate as u32 as i32),
            OpKind::Immediate64 => immediate as i64,
            _ => return None,
        };
        (signed < 0 || signed as u64 >= LARGE_CONSTANT).then_some(signed as u64)
    }
}
