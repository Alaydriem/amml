use std::collections::BTreeMap;

use iced_x86::{Decoder, DecoderOptions, FlowControl, Formatter, Instruction, OpKind};
use memchr::{memchr_iter, memmem};

use super::FunctionListing;
use crate::structs::{CodeRef, CodeRefKind, Image};

// The immediate sizes an instruction with a RIP-relative operand can have after the displacement
const IMMEDIATE_LENS: [u32; 4] = [0, 1, 2, 4];

/// Finds the code and data that refer to an address.
///
/// A byte search proposes candidate positions: `E8`, `E9` and `0F 8x` with a 32-bit displacement
/// that lands on the address, and a ModRM byte of the RIP-relative form whose displacement lands on
/// it. A candidate is reported only when decoding its function from the function start gives an
/// instruction that holds the position and names the address.
///
/// Known limits: jumps with an 8-bit displacement are not searched, a candidate outside every function
/// range is not reported, and data pointers are searched only in non-executable sections.
pub struct CodeXref<'a> {
    image: &'a Image,
}

impl<'a> CodeXref<'a> {
    pub fn new(image: &'a Image) -> Self {
        Self { image }
    }

    /// Every call, jump, RIP-relative operand and 8-byte data pointer that names `rva`, sorted by site.
    pub fn refs_to(&self, rva: u32) -> Vec<CodeRef> {
        let mut refs = self.code_refs(rva, true);
        refs.extend(self.data_pointers(rva));
        refs.sort_by_key(|reference| reference.site);
        refs
    }

    /// Every direct call and jump to `rva`, sorted by site.
    pub fn callers(&self, rva: u32) -> Vec<CodeRef> {
        self.code_refs(rva, false)
    }

    fn code_refs(&self, rva: u32, with_operands: bool) -> Vec<CodeRef> {
        let mut candidates = Vec::new();
        for section in self.image.sections().iter().filter(|s| s.executable) {
            Self::branch_candidates(section.rva, &section.data, rva, &mut candidates);
            if with_operands {
                Self::operand_candidates(section.rva, &section.data, rva, &mut candidates);
            }
        }
        candidates.sort_unstable();
        candidates.dedup();

        let mut by_range: BTreeMap<(u32, u32, u32), Vec<u32>> = BTreeMap::new();
        for candidate in candidates {
            if let Some(range) = self.range_of(candidate) {
                by_range.entry(range).or_default().push(candidate);
            }
        }

        let mut refs = Vec::new();
        for (range, positions) in by_range {
            self.confirm(rva, range, &positions, with_operands, &mut refs);
        }
        refs.sort_by_key(|reference| reference.site);
        refs
    }

    // (primary start, range start, range end) of the function range that holds `rva`
    fn range_of(&self, rva: u32) -> Option<(u32, u32, u32)> {
        let functions = self.image.functions();
        let (primary, end) = functions.containing(rva)?;
        let start = functions
            .ranges_of(primary)
            .into_iter()
            .find(|&(start, range_end)| start <= rva && rva < range_end)
            .map_or(primary, |(start, _)| start);
        Some((primary, start, end))
    }

    fn branch_candidates(base: u32, data: &[u8], target: u32, out: &mut Vec<u32>) {
        for at in memchr_iter(0xE8, data).chain(memchr_iter(0xE9, data)) {
            if data.get(at + 1..at + 5).is_some_and(|bytes| Self::lands(base, at, 5, bytes, target)) {
                out.extend(Self::rva_at(base, at));
            }
        }
        for at in memchr_iter(0x0F, data) {
            if !data.get(at + 1).is_some_and(|op| (0x80..=0x8F).contains(op)) {
                continue;
            }
            if data.get(at + 2..at + 6).is_some_and(|bytes| Self::lands(base, at, 6, bytes, target)) {
                out.extend(Self::rva_at(base, at));
            }
        }
    }

    fn operand_candidates(base: u32, data: &[u8], target: u32, out: &mut Vec<u32>) {
        for at in 1..data.len().saturating_sub(3) {
            // mod 00, rm 101: the four bytes after this ModRM byte are a RIP-relative displacement
            if data[at - 1] & 0xC7 != 0x05 {
                continue;
            }
            let disp = i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
            let Some(disp_rva) = Self::rva_at(base, at) else { continue };
            let hit = IMMEDIATE_LENS.iter().any(|immediate| {
                disp_rva.checked_add(4 + immediate).and_then(|next| next.checked_add_signed(disp)) == Some(target)
            });
            if hit {
                out.push(disp_rva);
            }
        }
    }

    // true when the instruction of `len` bytes at `at` with the displacement `bytes` reaches `target`
    fn lands(base: u32, at: usize, len: u32, bytes: &[u8], target: u32) -> bool {
        let disp = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Self::rva_at(base, at).and_then(|site| site.checked_add(len)).and_then(|next| next.checked_add_signed(disp))
            == Some(target)
    }

    fn rva_at(base: u32, offset: usize) -> Option<u32> {
        u32::try_from(offset).ok().and_then(|offset| base.checked_add(offset))
    }

    // Decodes one range from its start and keeps each instruction that holds a candidate and names `target`.
    // A byte that does not decode ends the range: nothing after it can be confirmed.
    fn confirm(
        &self,
        target: u32,
        (primary, start, end): (u32, u32, u32),
        positions: &[u32],
        with_operands: bool,
        out: &mut Vec<CodeRef>,
    ) {
        let Some(bytes) = self.image.read(start, (end - start) as usize) else { return };
        let mut decoder = Decoder::with_ip(64, bytes, u64::from(start), DecoderOptions::NONE);
        let mut formatter = FunctionListing::formatter();
        let mut instruction = Instruction::default();
        while decoder.can_decode() {
            decoder.decode_out(&mut instruction);
            if instruction.is_invalid() {
                return;
            }
            let site = instruction.ip() as u32;
            let first = positions.partition_point(|&position| position < site);
            if !positions.get(first).is_some_and(|&position| position < site + instruction.len() as u32) {
                continue;
            }
            let Some(kind) = Self::kind_of(&instruction, u64::from(target)) else { continue };
            if !with_operands && kind == CodeRefKind::RipRelative {
                continue;
            }
            let mut text = String::new();
            formatter.format(&instruction, &mut text);
            out.push(CodeRef { site, function: Some(primary), kind, text });
        }
    }

    fn kind_of(instruction: &Instruction, target: u64) -> Option<CodeRefKind> {
        if instruction.op0_kind() == OpKind::NearBranch64 && instruction.near_branch_target() == target {
            return match instruction.flow_control() {
                FlowControl::Call => Some(CodeRefKind::Call),
                FlowControl::UnconditionalBranch | FlowControl::ConditionalBranch => Some(CodeRefKind::Jump),
                _ => None,
            };
        }
        (instruction.is_ip_rel_memory_operand() && instruction.ip_rel_memory_address() == target)
            .then_some(CodeRefKind::RipRelative)
    }

    /// Every 8-byte, 8-aligned data pointer to `rva` in the non-executable sections.
    pub fn data_pointers(&self, rva: u32) -> Vec<CodeRef> {
        let pointer = self.image.image_base() + u64::from(rva);
        let finder = memmem::Finder::new(&pointer.to_le_bytes()).into_owned();
        let mut refs = Vec::new();
        for section in self.image.sections().iter().filter(|s| !s.executable) {
            for hit in finder.find_iter(&section.data) {
                let Some(site) = Self::rva_at(section.rva, hit) else { continue };
                if site % 8 != 0 {
                    continue;
                }
                refs.push(CodeRef {
                    site,
                    function: None,
                    kind: CodeRefKind::DataPointer,
                    text: format!("dq {pointer:#x}"),
                });
            }
        }
        refs
    }
}
