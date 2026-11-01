use amml_data::OperandKind;
use iced_x86::{Code, Decoder, DecoderOptions, Instruction, OpKind, Register};

use crate::errors::StepError;
use crate::structs::Image;

const MAX_INSTRUCTION_LEN: usize = 15;

pub struct Disasm;

impl Disasm {
    /// Reads one property of operand `index` of the instruction at `rva`.
    pub fn operand(image: &Image, rva: u32, index: u32, kind: OperandKind) -> Result<u64, StepError> {
        let instruction = Self::decode_at(image, rva)?;
        let no_operand = StepError::NoOperand { rva, index };
        if index >= instruction.op_count() {
            return Err(no_operand);
        }
        let op_kind = instruction.op_kind(index);
        match kind {
            // RIP-relative operands have no displacement (use RipTarget), and a negative one is not a field offset
            OperandKind::Displacement
                if op_kind == OpKind::Memory
                    && !instruction.is_ip_rel_memory_operand()
                    && (instruction.memory_displacement64() as i64) >= 0 =>
            {
                Ok(instruction.memory_displacement64())
            }
            OperandKind::Immediate if Self::is_immediate(op_kind) => Ok(instruction.immediate(index)),
            OperandKind::RipTarget if op_kind == OpKind::Memory && instruction.is_ip_rel_memory_operand() => {
                Ok(instruction.ip_rel_memory_address())
            }
            OperandKind::BranchTarget if op_kind == OpKind::NearBranch64 => Ok(instruction.near_branch_target()),
            OperandKind::ScaleFactor if op_kind == OpKind::Memory && instruction.memory_index() != Register::None => {
                let scale = u64::from(instruction.memory_index_scale());
                Ok(if instruction.memory_base() == instruction.memory_index() { scale + 1 } else { scale })
            }
            _ => Err(no_operand),
        }
    }

    /// The member offset that memory operand `index` of the instruction at `rva` reads. Through an object pointer
    /// it is the displacement. Through the frame (rbp or rsp) the object is a local: the offset is the displacement
    /// minus that of memory operand 1 of the instruction at `rva + local_at` (the `lea` of the local).
    pub fn member_displacement(image: &Image, rva: u32, index: u32, local_at: i64) -> Result<u64, StepError> {
        let no_operand = StepError::NoOperand { rva, index };
        // `add rsi,30h` forms the address of a member from the object pointer: the immediate is the offset
        let instruction = Self::decode_at(image, rva)?;
        if index < instruction.op_count() && Self::is_immediate(instruction.op_kind(index)) {
            return u64::try_from(instruction.immediate(index) as i64).map_err(|_| no_operand);
        }
        let (near, framed) = Self::memory_displacement(image, rva, index)?;
        let offset = if framed {
            let local = u32::try_from(i64::from(rva) + local_at)
                .map_err(|_| StepError::Overflow { value: u64::from(rva), delta: local_at })?;
            let (far, _) = Self::memory_displacement(image, local, 1)?;
            near.checked_sub(far)
        } else {
            Some(near)
        };
        offset.and_then(|value| u64::try_from(value).ok()).ok_or(no_operand)
    }

    // the signed displacement of memory operand `index`, and whether its base is the frame (rbp or rsp)
    fn memory_displacement(image: &Image, rva: u32, index: u32) -> Result<(i64, bool), StepError> {
        let instruction = Self::decode_at(image, rva)?;
        if index >= instruction.op_count()
            || instruction.op_kind(index) != OpKind::Memory
            || instruction.is_ip_rel_memory_operand()
        {
            return Err(StepError::NoOperand { rva, index });
        }
        let framed = matches!(instruction.memory_base(), Register::RBP | Register::RSP);
        Ok((instruction.memory_displacement64() as i64, framed))
    }

    /// Target RVA of the `nth` (0-based) direct `call rel32` in the function containing `rva`,
    /// counted across the function's ranges in ascending order.
    pub fn nth_call(image: &Image, rva: u32, nth: u32) -> Result<u32, StepError> {
        let (primary, _) = image.functions().containing(rva).ok_or(StepError::NotInFunction(rva))?;
        let mut found = 0u32;
        for (start, end) in image.functions().ranges_of(primary) {
            let len = end.checked_sub(start).ok_or(StepError::BadInstruction(start))?;
            let bytes = image.read(start, len as usize).ok_or(StepError::BadInstruction(start))?;
            let mut decoder = Decoder::with_ip(64, bytes, u64::from(start), DecoderOptions::NONE);
            let mut instruction = Instruction::default();
            while decoder.can_decode() {
                decoder.decode_out(&mut instruction);
                if instruction.is_invalid() {
                    return Err(StepError::BadInstruction(instruction.ip() as u32));
                }
                if instruction.code() != Code::Call_rel32_64 {
                    continue;
                }
                if found == nth {
                    return u32::try_from(instruction.near_branch_target())
                        .map_err(|_| StepError::BadInstruction(instruction.ip() as u32));
                }
                found += 1;
            }
        }
        Err(StepError::CallIndex { nth, found })
    }

    fn is_immediate(kind: OpKind) -> bool {
        matches!(
            kind,
            OpKind::Immediate8
                | OpKind::Immediate8_2nd
                | OpKind::Immediate16
                | OpKind::Immediate32
                | OpKind::Immediate64
                | OpKind::Immediate8to16
                | OpKind::Immediate8to32
                | OpKind::Immediate8to64
                | OpKind::Immediate32to64
        )
    }

    fn decode_at(image: &Image, rva: u32) -> Result<Instruction, StepError> {
        let section = image.section_of(rva).ok_or(StepError::BadInstruction(rva))?;
        let offset = (rva - section.rva) as usize;
        let end = section.data.len().min(offset + MAX_INSTRUCTION_LEN);
        let bytes = section.data.get(offset..end).ok_or(StepError::BadInstruction(rva))?;
        let mut decoder = Decoder::with_ip(64, bytes, u64::from(rva), DecoderOptions::NONE);
        let instruction = decoder.decode();
        if instruction.is_invalid() {
            return Err(StepError::BadInstruction(rva));
        }
        Ok(instruction)
    }
}
