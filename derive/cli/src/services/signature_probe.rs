use std::collections::{BTreeMap, BTreeSet};

use iced_x86::{
    Code, Decoder, DecoderOptions, FlowControl, Formatter, Instruction, InstructionInfoFactory, IntelFormatter,
    Mnemonic, OpAccess, OpKind, Register,
};

use crate::structs::{Image, Reg, ReturnKind, Signature};

// what is known at one instruction about the return registers, written since entry or since the last call
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReturnState {
    rax: bool,
    xmm0: bool,
    // the one written last, when every path agrees
    last: Option<ReturnKind>,
}

impl ReturnState {
    const EMPTY: Self = Self { rax: false, xmm0: false, last: None };

    fn meet(self, other: Self) -> Self {
        Self {
            rax: self.rax && other.rax,
            xmm0: self.xmm0 && other.xmm0,
            last: if self.last == other.last { self.last } else { None },
        }
    }
}

pub struct SignatureProbe<'a> {
    image: &'a Image,
}

impl<'a> SignatureProbe<'a> {
    /// How far a function without a `.pdata` range is decoded looking for its `ret` or `jmp`.
    pub const MAX_THUNK_LEN: u32 = 0x100;
    /// The first incoming stack argument, past the return address and the 0x20 bytes of home space.
    pub const FIRST_STACK_ARG: i64 = 0x28;

    pub fn new(image: &'a Image) -> Self {
        Self { image }
    }

    /// The signature of the function that starts at `rva`. Any part the probe cannot follow makes it unknown.
    pub fn probe(&self, rva: u32) -> Signature {
        self.try_probe(rva).unwrap_or_else(Signature::unknown)
    }

    fn try_probe(&self, rva: u32) -> Result<Signature, String> {
        let (scan, body) = self.code_of(rva)?;
        let mut factory = InstructionInfoFactory::new();
        let (stack_args, depths) = Self::stack_reads(&scan, &mut factory)?;
        let (args_int, args_xmm) = Self::argument_reads(&scan, &depths, &mut factory)?;
        let returns = Self::return_kind(rva, &body, &mut factory);
        Ok(Signature::new(args_int, args_xmm, stack_args, returns))
    }

    // The instructions from `rva` to the end of its `.pdata` range, and those of every range of the function for the
    // return analysis. Without a range, a leaf thunk is decoded up to its first `ret` or `jmp`.
    fn code_of(&self, rva: u32) -> Result<(Vec<Instruction>, Vec<Instruction>), String> {
        let functions = self.image.functions();
        let Some((primary, _)) = functions.containing(rva) else {
            let thunk = self.decode_thunk(rva)?;
            return Ok((thunk.clone(), thunk));
        };
        let ranges = functions.ranges_of(primary);
        let (_, end) = ranges
            .iter()
            .copied()
            .find(|(from, to)| (*from..*to).contains(&rva))
            .ok_or_else(|| format!("no range of the function at {primary:#x} holds {rva:#x}"))?;
        let scan = self.decode_range(rva, end)?;
        if rva != primary {
            return Ok((scan.clone(), scan));
        }
        let mut body = Vec::new();
        for (from, to) in ranges {
            body.extend(self.decode_range(from, to)?);
        }
        Ok((scan, body))
    }

    // every instruction of `[from, to)`; the range has to end right after a `ret`, a jump, a trap or a call
    fn decode_range(&self, from: u32, to: u32) -> Result<Vec<Instruction>, String> {
        let len = to.checked_sub(from).ok_or_else(|| format!("range {from:#x}..{to:#x} is empty"))?;
        let bytes = self
            .image
            .read(from, len as usize)
            .ok_or_else(|| format!("range {from:#x}..{to:#x} leaves its section"))?;
        let mut decoder = Decoder::with_ip(64, bytes, u64::from(from), DecoderOptions::NONE);
        let mut instructions = Vec::new();
        while decoder.can_decode() {
            let instruction = decoder.decode();
            if instruction.is_invalid() {
                return Err(format!(
                    "cannot decode at {:#x} in range {from:#x}..{to:#x} (the range may be cut)",
                    instruction.ip()
                ));
            }
            instructions.push(instruction);
        }
        match instructions.last() {
            Some(last) if Self::ends_code(last) => Ok(instructions),
            Some(last) => Err(format!(
                "range {from:#x}..{to:#x} ends after `{}` at {:#x}, inside code (the range may be cut)",
                Self::text(last),
                last.ip()
            )),
            None => Err(format!("range {from:#x}..{to:#x} is empty")),
        }
    }

    fn decode_thunk(&self, rva: u32) -> Result<Vec<Instruction>, String> {
        let section = self.image.section_of(rva).ok_or_else(|| format!("{rva:#x} is in no section"))?;
        let offset = (rva - section.rva) as usize;
        let end = section.data.len().min(offset + Self::MAX_THUNK_LEN as usize);
        let bytes = section.data.get(offset..end).ok_or_else(|| format!("{rva:#x} is past its section data"))?;
        let mut decoder = Decoder::with_ip(64, bytes, u64::from(rva), DecoderOptions::NONE);
        let mut instructions = Vec::new();
        while decoder.can_decode() {
            let instruction = decoder.decode();
            if instruction.is_invalid() {
                return Err(format!("cannot decode at {:#x} in the thunk at {rva:#x}", instruction.ip()));
            }
            instructions.push(instruction);
            if matches!(
                instruction.flow_control(),
                FlowControl::Return | FlowControl::UnconditionalBranch | FlowControl::IndirectBranch
            ) {
                return Ok(instructions);
            }
        }
        Err(format!(
            "no .pdata range holds {rva:#x}, and no ret or jmp follows within {:#x} bytes",
            Self::MAX_THUNK_LEN
        ))
    }

    fn text(instruction: &Instruction) -> String {
        let mut text = String::new();
        IntelFormatter::new().format(instruction, &mut text);
        text
    }

    fn ends_code(instruction: &Instruction) -> bool {
        !matches!(
            instruction.flow_control(),
            FlowControl::Next | FlowControl::ConditionalBranch | FlowControl::XbeginXabortXend
        )
    }

    // The argument registers read before they are written, from the start up to the first call, `ret`, trap or
    // unconditional jump out of the scanned range. A conditional jump never stops the scan; it goes on along the
    // fall-through. An indirect jump ends it only as a tail jump: after an argument read, with the stack back at its
    // entry value, and with nothing but padding after it. Any other indirect jump makes the signature unknown.
    // `depths[i]` is the stack depth before `scan[i]`.
    fn argument_reads(
        scan: &[Instruction],
        depths: &[i64],
        factory: &mut InstructionInfoFactory,
    ) -> Result<(Vec<Reg>, Vec<Reg>), String> {
        let (Some(first), Some(last)) = (scan.first(), scan.last()) else { return Err("no code".to_string()) };
        let range = first.ip()..last.next_ip();
        let mut reads: [Option<u32>; 8] = [None; 8];
        let mut written = [false; 8];
        for (index, instruction) in scan.iter().enumerate() {
            let merged = Self::merge_source(instruction);
            let info = factory.info(instruction);
            for used in info.used_registers() {
                let Some(slot) = Self::argument_slot(used.register()) else { continue };
                if Some(used.register()) == merged {
                    continue;
                }
                if Self::is_read(used.access()) && !written[slot] && reads[slot].is_none() {
                    reads[slot] = Some(used.register().size() as u32 * 8);
                }
            }
            for used in info.used_registers() {
                let Some(slot) = Self::argument_slot(used.register()) else { continue };
                if matches!(used.access(), OpAccess::Write | OpAccess::ReadWrite) {
                    written[slot] = true;
                }
            }
            let stop = match instruction.flow_control() {
                FlowControl::Call | FlowControl::IndirectCall => !Self::is_stack_probe(scan, index),
                FlowControl::Return | FlowControl::Interrupt | FlowControl::Exception => true,
                FlowControl::IndirectBranch => {
                    let ip = instruction.ip();
                    if reads.iter().all(Option::is_none) {
                        return Err(format!("indirect jump at {ip:#x} before any argument read"));
                    }
                    let depth = depths.get(index).copied().unwrap_or(-1);
                    if depth != 0 {
                        return Err(format!(
                            "indirect jump at {ip:#x} with the stack {depth:#x} bytes below its entry value, so not a tail jump"
                        ));
                    }
                    if scan.get(index + 1..).unwrap_or_default().iter().any(|after| !Self::is_padding(after)) {
                        return Err(format!(
                            "indirect jump at {ip:#x} has code after it (a jump table?), so not a tail jump"
                        ));
                    }
                    true
                }
                FlowControl::UnconditionalBranch => {
                    !Self::is_near_branch(instruction) || !range.contains(&instruction.near_branch_target())
                }
                _ => false,
            };
            if stop {
                break;
            }
        }
        let regs: Vec<Reg> = Reg::ARGUMENTS
            .iter()
            .zip(reads)
            .filter_map(|(register, bits)| bits.map(|bits| Reg::new(*register, bits)))
            .collect();
        Ok(regs.into_iter().partition(|reg| !reg.is_xmm()))
    }

    fn is_padding(instruction: &Instruction) -> bool {
        matches!(instruction.mnemonic(), Mnemonic::Int3 | Mnemonic::Nop)
    }

    // `mov eax,size; call __chkstk; sub rsp,rax`: the probe keeps every register but rax, r10 and r11
    fn is_stack_probe(scan: &[Instruction], index: usize) -> bool {
        let before = index.checked_sub(1).and_then(|before| scan.get(before));
        let after = scan.get(index + 1);
        matches!(before, Some(mov) if mov.code() == Code::Mov_r32_imm32 && mov.op0_register() == Register::EAX)
            && matches!(after, Some(sub) if Self::is_sub_rsp_rax(sub))
    }

    fn is_sub_rsp_rax(instruction: &Instruction) -> bool {
        matches!(instruction.code(), Code::Sub_rm64_r64 | Code::Sub_r64_rm64)
            && instruction.op0_kind() == OpKind::Register
            && instruction.op0_register() == Register::RSP
            && instruction.op1_kind() == OpKind::Register
            && instruction.op1_register() == Register::RAX
    }

    // The incoming stack slots read anywhere in the scanned range, as offsets from the stack pointer at entry. The
    // stack pointer is followed through pushes, `sub`/`add rsp,imm`, the stack probe and an rbp frame; after a `ret`
    // or an unconditional jump it is back at the body's frame. Any other change to rsp makes the result unknown.
    // Also gives the depth before each instruction.
    fn stack_reads(scan: &[Instruction], factory: &mut InstructionInfoFactory) -> Result<(Vec<u32>, Vec<i64>), String> {
        let mut slots = BTreeSet::new();
        let mut depths = Vec::with_capacity(scan.len());
        // bytes below the entry stack pointer
        let mut depth: i64 = 0;
        let mut frame_depth: i64 = 0;
        // depth of the address in rbp, when rbp is a frame pointer
        let mut rbp: Option<i64> = None;
        let mut frame_rbp: Option<i64> = None;
        let mut eax: Option<i64> = None;
        for instruction in scan {
            depths.push(depth);
            let info = factory.info(instruction);
            for memory in info.used_memory() {
                if !Self::is_read(memory.access()) || memory.index() != Register::None {
                    continue;
                }
                let base = match memory.base() {
                    Register::RSP => Some(depth),
                    Register::RBP => rbp,
                    _ => None,
                };
                if let Some(base) = base {
                    let offset = (memory.displacement() as i64).wrapping_sub(base);
                    if offset >= Self::FIRST_STACK_ARG {
                        slots.insert(u32::try_from(offset).map_err(|_| format!("stack read at {offset:#x}"))?);
                    }
                }
            }
            let writes = |register: Register| {
                info.used_registers().iter().any(|used| {
                    used.register().full_register() == register.full_register()
                        && matches!(used.access(), OpAccess::Write | OpAccess::ReadWrite | OpAccess::CondWrite)
                })
            };
            let writes_rsp = writes(Register::RSP);
            let writes_rbp = writes(Register::RBP);
            let writes_rax = writes(Register::RAX);
            let flow = instruction.flow_control();
            let code = instruction.code();
            let rsp_dest = instruction.op0_kind() == OpKind::Register && instruction.op0_register() == Register::RSP;
            if matches!(flow, FlowControl::Call | FlowControl::IndirectCall | FlowControl::Return) {
                // the callee pops what the call pushes
            } else if rsp_dest && matches!(code, Code::Sub_rm64_imm8 | Code::Sub_rm64_imm32) {
                depth += instruction.immediate(1) as i64;
            } else if rsp_dest && matches!(code, Code::Add_rm64_imm8 | Code::Add_rm64_imm32) {
                depth -= instruction.immediate(1) as i64;
            } else if Self::is_sub_rsp_rax(instruction) {
                depth += eax.ok_or_else(|| format!("sub rsp,rax at {:#x} without a known eax", instruction.ip()))?;
            } else if rsp_dest && Self::is_frame_copy(instruction, Register::RBP) {
                let base = rbp.ok_or_else(|| format!("rsp set from rbp at {:#x} without a frame", instruction.ip()))?;
                depth = base - Self::frame_offset(instruction);
            } else if instruction.stack_pointer_increment() != 0 {
                depth -= i64::from(instruction.stack_pointer_increment());
            } else if writes_rsp {
                return Err(format!("rsp changes at {:#x} in a way the probe does not follow", instruction.ip()));
            }
            if instruction.op0_kind() == OpKind::Register
                && instruction.op0_register() == Register::RBP
                && Self::is_frame_copy(instruction, Register::RSP)
            {
                rbp = Some(depth - Self::frame_offset(instruction));
                frame_rbp = rbp;
            } else if writes_rbp {
                rbp = None;
            }
            eax = match (code, writes_rax) {
                (Code::Mov_r32_imm32, _) if instruction.op0_register() == Register::EAX => {
                    Some(i64::from(instruction.immediate32()))
                }
                (_, true) => None,
                _ => eax,
            };
            frame_depth = frame_depth.max(depth);
            if matches!(flow, FlowControl::Return | FlowControl::UnconditionalBranch | FlowControl::IndirectBranch) {
                depth = frame_depth;
                rbp = frame_rbp;
            }
        }
        Ok((slots.into_iter().collect(), depths))
    }

    // `mov dst,src` or `lea dst,[src+disp]`
    fn is_frame_copy(instruction: &Instruction, source: Register) -> bool {
        match instruction.code() {
            Code::Mov_rm64_r64 | Code::Mov_r64_rm64 => {
                instruction.op1_kind() == OpKind::Register && instruction.op1_register() == source
            }
            Code::Lea_r64_m => instruction.memory_base() == source && instruction.memory_index() == Register::None,
            _ => false,
        }
    }

    fn frame_offset(instruction: &Instruction) -> i64 {
        if instruction.code() == Code::Lea_r64_m { instruction.memory_displacement64() as i64 } else { 0 }
    }

    // Must-analysis over the function's control flow: at every `ret`, which of rax and xmm0 this function wrote
    // since entry or since its last call. Code that no direct edge reaches (behind a jump table) starts empty.
    fn return_kind(entry: u32, body: &[Instruction], factory: &mut InstructionInfoFactory) -> ReturnKind {
        let index: BTreeMap<u64, usize> =
            body.iter().enumerate().map(|(position, instruction)| (instruction.ip(), position)).collect();
        let mut states: Vec<Option<ReturnState>> = vec![None; body.len()];
        let mut at_returns: Vec<Option<ReturnState>> = vec![None; body.len()];
        let mut pending: Vec<usize> = Vec::new();
        if let Some(start) = index.get(&u64::from(entry)) {
            states[*start] = Some(ReturnState::EMPTY);
            pending.push(*start);
        }
        loop {
            while let Some(position) = pending.pop() {
                let (Some(instruction), Some(state)) = (body.get(position), states[position]) else { continue };
                if instruction.flow_control() == FlowControl::Return {
                    at_returns[position] = Some(state);
                    continue;
                }
                let out = Self::transfer(instruction, state, factory);
                for next in Self::successors(body, &index, position) {
                    let merged = states[next].map_or(out, |old| old.meet(out));
                    if states[next] != Some(merged) {
                        states[next] = Some(merged);
                        pending.push(next);
                    }
                }
            }
            match states.iter().position(Option::is_none) {
                Some(unreached) => {
                    states[unreached] = Some(ReturnState::EMPTY);
                    pending.push(unreached);
                }
                None => break,
            }
        }
        let returns: Vec<ReturnState> = at_returns.into_iter().flatten().collect();
        if returns.is_empty() {
            return ReturnKind::None;
        }
        let all_rax = returns.iter().all(|state| state.rax);
        let all_xmm0 = returns.iter().all(|state| state.xmm0);
        let last = returns.iter().map(|state| state.last).reduce(|a, b| if a == b { a } else { None }).flatten();
        match (all_rax, all_xmm0) {
            (true, true) => last.unwrap_or(ReturnKind::Int),
            (true, false) => ReturnKind::Int,
            (false, true) => ReturnKind::Float,
            (false, false) => ReturnKind::None,
        }
    }

    fn transfer(instruction: &Instruction, state: ReturnState, factory: &mut InstructionInfoFactory) -> ReturnState {
        if matches!(instruction.flow_control(), FlowControl::Call | FlowControl::IndirectCall) {
            return ReturnState::EMPTY;
        }
        let mut out = state;
        for used in factory.info(instruction).used_registers() {
            if !matches!(used.access(), OpAccess::Write | OpAccess::ReadWrite) {
                continue;
            }
            let register = used.register();
            if register.is_gpr() && register.full_register() == Register::RAX {
                out.rax = true;
                out.last = Some(ReturnKind::Int);
            } else if register.is_vector_register() && register.number() == 0 {
                out.xmm0 = true;
                out.last = Some(ReturnKind::Float);
            }
        }
        out
    }

    fn successors(body: &[Instruction], index: &BTreeMap<u64, usize>, position: usize) -> Vec<usize> {
        let Some(instruction) = body.get(position) else { return Vec::new() };
        let fall_through =
            body.get(position + 1).filter(|next| next.ip() == instruction.next_ip()).map(|_| position + 1);
        let target =
            Self::is_near_branch(instruction).then(|| index.get(&instruction.near_branch_target()).copied()).flatten();
        match instruction.flow_control() {
            FlowControl::Return | FlowControl::IndirectBranch | FlowControl::Interrupt | FlowControl::Exception => {
                Vec::new()
            }
            FlowControl::UnconditionalBranch => target.into_iter().collect(),
            FlowControl::ConditionalBranch => fall_through.into_iter().chain(target).collect(),
            _ => fall_through.into_iter().collect(),
        }
    }

    fn is_near_branch(instruction: &Instruction) -> bool {
        matches!(instruction.op0_kind(), OpKind::NearBranch16 | OpKind::NearBranch32 | OpKind::NearBranch64)
    }

    // 0 to 3 for rcx, rdx, r8 and r9 in any width; 4 to 7 for xmm0 to xmm3 (and their ymm and zmm forms)
    fn argument_slot(register: Register) -> Option<usize> {
        if register.is_gpr() {
            return match register.full_register() {
                Register::RCX => Some(0),
                Register::RDX => Some(1),
                Register::R8 => Some(2),
                Register::R9 => Some(3),
                _ => None,
            };
        }
        (register.is_vector_register() && register.number() < 4).then(|| 4 + register.number())
    }

    // A scalar SSE operation that writes the low element of its destination and keeps the rest of the register: the
    // register the rest comes from is read, but not as a value. That is the destination of the legacy form and the
    // first source of the VEX form, when no other operand names it.
    fn merge_source(instruction: &Instruction) -> Option<Register> {
        let legacy = matches!(
            instruction.mnemonic(),
            Mnemonic::Movss
                | Mnemonic::Movsd
                | Mnemonic::Cvtsi2ss
                | Mnemonic::Cvtsi2sd
                | Mnemonic::Cvtss2sd
                | Mnemonic::Cvtsd2ss
                | Mnemonic::Sqrtss
                | Mnemonic::Sqrtsd
                | Mnemonic::Rcpss
                | Mnemonic::Rsqrtss
                | Mnemonic::Roundss
                | Mnemonic::Roundsd
        );
        let vex = matches!(
            instruction.mnemonic(),
            Mnemonic::Vmovss
                | Mnemonic::Vmovsd
                | Mnemonic::Vcvtsi2ss
                | Mnemonic::Vcvtsi2sd
                | Mnemonic::Vcvtss2sd
                | Mnemonic::Vcvtsd2ss
                | Mnemonic::Vsqrtss
                | Mnemonic::Vsqrtsd
                | Mnemonic::Vrcpss
                | Mnemonic::Vrsqrtss
                | Mnemonic::Vroundss
                | Mnemonic::Vroundsd
        );
        let register_at = |operand: u32| {
            (instruction.op_count() > operand && instruction.op_kind(operand) == OpKind::Register)
                .then(|| instruction.op_register(operand))
                .filter(|register| register.is_xmm())
        };
        let (merged, others) = if legacy && instruction.op_count() >= 2 {
            // `movss xmm,[mem]` and `movsd xmm,[mem]` zero the rest instead
            let loads = matches!(instruction.mnemonic(), Mnemonic::Movss | Mnemonic::Movsd)
                && instruction.op1_kind() != OpKind::Register;
            (register_at(0).filter(|_| !loads), 1..instruction.op_count())
        } else if vex && instruction.op_count() >= 3 {
            (register_at(1), 2..instruction.op_count())
        } else {
            return None;
        };
        let merged = merged?;
        others.into_iter().all(|operand| register_at(operand) != Some(merged)).then_some(merged)
    }

    fn is_read(access: OpAccess) -> bool {
        matches!(access, OpAccess::Read | OpAccess::CondRead | OpAccess::ReadWrite | OpAccess::ReadCondWrite)
    }
}
