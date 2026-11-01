use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, IntelFormatter, OpKind};

use crate::errors::StepError;
use crate::structs::{CoveredInstruction, FieldKind, Image};

const MAX_INSTRUCTION_LEN: usize = 15;

/// Finds the bytes of code that are build-specific: immediates, displacements and branch offsets.
pub struct FixedFields<'a> {
    image: &'a Image,
}

impl<'a> FixedFields<'a> {
    pub fn new(image: &'a Image) -> Self {
        Self { image }
    }

    /// The RVA of every byte that sits in a field, with its kind, for the whole instructions that cover
    /// `[start, start + len)`. The decode starts at the function start, `start` must be an instruction start, and
    /// the range ends at the end of the function range that holds `start`.
    pub fn fields(&self, start: u32, len: usize) -> Result<Vec<(u32, FieldKind)>, StepError> {
        let mut found = Vec::new();
        for instruction in self.instructions(start, len)? {
            for (kind, offset, size) in &instruction.fields {
                for at in 0..*size {
                    found.push((instruction.ip + (*offset + at) as u32, *kind));
                }
            }
        }
        Ok(found)
    }

    /// The whole instructions from `start` that cover at least `len` bytes, or all that remain in the function
    /// range when it ends first.
    pub fn instructions(&self, start: u32, len: usize) -> Result<Vec<CoveredInstruction>, StepError> {
        let functions = self.image.functions();
        let (primary, _) = functions.containing(start).ok_or(StepError::NotInListedFunction(start))?;
        let range_end = functions
            .ranges_of(primary)
            .into_iter()
            .find(|(from, to)| (*from..*to).contains(&start))
            .map(|(_, to)| to)
            .ok_or(StepError::NotInListedFunction(start))?;
        let clamped = len.clamp(1, (range_end - start) as usize);
        let covering = self.cover(start, clamped, false).map_err(StepError::Undecodable)?;
        match covering.first() {
            Some(first) if first.ip == start => Ok(covering),
            _ => Err(StepError::NotInstructionStart(start)),
        }
    }

    /// Decodes from the start of the function range that holds `start` until past the match end, so an
    /// instruction the match starts inside is whole. A match in no `.pdata` range (a leaf without unwind data)
    /// is decoded from its own start, but only when `code_start` says the start is the first byte of code.
    /// Returns the instructions that overlap `[start, start + len)`.
    pub fn cover(&self, start: u32, len: usize, code_start: bool) -> Result<Vec<CoveredInstruction>, String> {
        let outside = || "no .pdata range or section holds the match".to_string();
        let end = u32::try_from(len).ok().and_then(|len| start.checked_add(len)).ok_or("match end overflows")?;
        let functions = self.image.functions();
        let range_start = match functions.containing(start) {
            Some((primary, _)) => {
                functions
                    .ranges_of(primary)
                    .into_iter()
                    .find(|(from, to)| (*from..*to).contains(&start))
                    .ok_or_else(outside)?
                    .0
            }
            None if code_start => start,
            None => return Err(outside()),
        };
        let section = self.image.section_of(range_start).ok_or_else(outside)?;
        let from = (range_start - section.rva) as usize;
        let to = (end - section.rva) as usize + MAX_INSTRUCTION_LEN;
        let bytes = section.data.get(from..to.min(section.data.len())).ok_or_else(outside)?;
        let mut decoder = Decoder::with_ip(64, bytes, u64::from(range_start), DecoderOptions::NONE);
        let mut formatter = IntelFormatter::new();
        let mut instruction = Instruction::default();
        let mut covering = Vec::new();
        loop {
            if !decoder.can_decode() {
                // the match may end exactly where the section data ends
                if decoder.ip() >= u64::from(end) {
                    return Ok(covering);
                }
                return Err(format!("section data ends before the match at {start:#x}"));
            }
            decoder.decode_out(&mut instruction);
            let ip = instruction.ip() as u32;
            if instruction.is_invalid() {
                return Err(format!("no valid instruction at {ip:#x} on the way to the match at {start:#x}"));
            }
            if ip >= end {
                return Ok(covering);
            }
            if ip + instruction.len() as u32 <= start {
                continue;
            }
            let mut text = String::new();
            formatter.format(&instruction, &mut text);
            covering.push(CoveredInstruction {
                ip,
                len: instruction.len(),
                text,
                fields: Self::instruction_fields(&decoder, &instruction),
            });
        }
    }

    fn instruction_fields(decoder: &Decoder, instruction: &Instruction) -> Vec<(FieldKind, usize, usize)> {
        let offsets = decoder.get_constant_offsets(instruction);
        let mut fields = Vec::new();
        if offsets.has_displacement() {
            let rip = instruction.is_ip_rel_memory_operand();
            let kind = if rip { FieldKind::RipDisplacement } else { FieldKind::Displacement };
            fields.push((kind, offsets.displacement_offset(), offsets.displacement_size()));
        }
        if offsets.has_immediate() {
            let kind = if Self::is_relative_branch(instruction) { FieldKind::Branch } else { FieldKind::Immediate };
            fields.push((kind, offsets.immediate_offset(), offsets.immediate_size()));
        }
        if offsets.has_immediate2() {
            fields.push((FieldKind::Immediate, offsets.immediate_offset2(), offsets.immediate_size2()));
        }
        fields
    }

    // jmp, jcc, call, loop and jrcxz carry a near branch operand; the immediate field then holds the distance
    fn is_relative_branch(instruction: &Instruction) -> bool {
        matches!(instruction.op0_kind(), OpKind::NearBranch16 | OpKind::NearBranch32 | OpKind::NearBranch64)
    }
}
