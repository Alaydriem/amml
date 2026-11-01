use std::collections::{BTreeMap, BTreeSet};

use iced_x86::{Decoder, DecoderOptions, FlowControl, InstructionInfoFactory, Mnemonic, OpAccess, OpKind, Register};

use crate::errors::StepError;
use crate::structs::{AccessKind, FieldAccessRow, Image};

// the registers a call may change (Windows x64)
const VOLATILE: [Register; 7] =
    [Register::RAX, Register::RCX, Register::RDX, Register::R8, Register::R9, Register::R10, Register::R11];

/// Lists the members of `this` that a function uses, and lines them up between two builds. A constructor, a
/// destructor or a copy uses most members in order, so a block of members that moved shows as one delta.
pub struct FieldAccess;

impl FieldAccess {
    /// Every memory operand whose base holds `this`, in function order. `this` is `rcx` at entry; a
    /// `mov r64, r64` from a register that holds it adds the destination, and any other write to such a register,
    /// or a call for the volatile registers, removes it. Operands with an index register or a negative
    /// displacement are left out.
    pub fn list(image: &Image, function: u32) -> Result<Vec<FieldAccessRow>, StepError> {
        let ranges = image.functions().ranges_of(function);
        if ranges.is_empty() {
            return Err(StepError::NotInFunction(function));
        }
        let mut tracked = BTreeSet::from([Register::RCX]);
        let mut info = InstructionInfoFactory::new();
        let mut rows = Vec::new();
        for (start, end) in ranges {
            let len = usize::try_from(end - start).map_err(|_| StepError::BadInstruction(start))?;
            let bytes = image.read(start, len).ok_or(StepError::BadInstruction(start))?;
            let mut decoder = Decoder::with_ip(64, bytes, u64::from(start), DecoderOptions::NONE);
            for instruction in &mut decoder {
                if instruction.is_invalid() {
                    break;
                }
                // `lea` computes an address and touches no memory, so the decoder lists no memory use for it
                if instruction.mnemonic() == Mnemonic::Lea
                    && instruction.op1_kind() == OpKind::Memory
                    && tracked.contains(&instruction.memory_base().full_register())
                    && instruction.memory_index() == Register::None
                    && let Ok(disp) = u32::try_from(instruction.memory_displacement64() as i64)
                {
                    let width = u8::try_from(instruction.op0_register().size()).unwrap_or(u8::MAX);
                    rows.push(FieldAccessRow { kind: AccessKind::Lea, disp, width, rva: instruction.ip32() });
                }
                let used = info.info(&instruction);
                for memory in used.used_memory() {
                    let displacement = memory.displacement() as i64;
                    if !tracked.contains(&memory.base().full_register())
                        || memory.index() != Register::None
                        || displacement < 0
                    {
                        continue;
                    }
                    let Ok(disp) = u32::try_from(displacement) else { continue };
                    let kind = match memory.access() {
                        OpAccess::NoMemAccess => continue,
                        OpAccess::Read | OpAccess::CondRead => AccessKind::Read,
                        _ => AccessKind::Write,
                    };
                    let width = memory.memory_size().size();
                    let width = u8::try_from(width).unwrap_or(u8::MAX);
                    rows.push(FieldAccessRow { kind, disp, width, rva: instruction.ip32() });
                }
                let written: Vec<Register> = used
                    .used_registers()
                    .iter()
                    .filter(|used| {
                        matches!(
                            used.access(),
                            OpAccess::Write | OpAccess::CondWrite | OpAccess::ReadWrite | OpAccess::ReadCondWrite
                        )
                    })
                    .map(|used| used.register().full_register())
                    .collect();
                for register in written {
                    tracked.remove(&register);
                }
                if matches!(instruction.flow_control(), FlowControl::Call | FlowControl::IndirectCall) {
                    for register in VOLATILE {
                        tracked.remove(&register);
                    }
                }
                let copies_this = instruction.mnemonic() == Mnemonic::Mov
                    && instruction.op0_kind() == OpKind::Register
                    && instruction.op1_kind() == OpKind::Register
                    && instruction.op0_register().size() == 8
                    && tracked.contains(&instruction.op1_register());
                if copies_this {
                    tracked.insert(instruction.op0_register());
                }
            }
        }
        Ok(rows)
    }

    /// Pairs the rows of two builds. Only rows of the same kind and width pair; the pairing has the most pairs, and
    /// among those the most pairs with the same offset, so a member that is new in one build stands alone. A row
    /// with no partner stands alone, in function order.
    pub fn align(
        old: &[FieldAccessRow],
        new: &[FieldAccessRow],
    ) -> Vec<(Option<FieldAccessRow>, Option<FieldAccessRow>)> {
        // a pair scores 2, and 1 more when the offset is the same
        let score = |a: &FieldAccessRow, b: &FieldAccessRow| {
            if (a.kind, a.width) != (b.kind, b.width) {
                None
            } else if a.disp == b.disp {
                Some(3u32)
            } else {
                Some(2u32)
            }
        };
        let (n, m) = (old.len(), new.len());
        // best[i][j]: the best score of old[i..] and new[j..]
        let mut best = vec![vec![0u32; m + 1]; n + 1];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                let skip = best[i + 1][j].max(best[i][j + 1]);
                best[i][j] = score(&old[i], &new[j]).map_or(skip, |pair| skip.max(best[i + 1][j + 1] + pair));
            }
        }
        let (mut i, mut j) = (0, 0);
        let mut pairs = Vec::with_capacity(n.max(m));
        while i < n && j < m {
            if score(&old[i], &new[j]).is_some_and(|pair| best[i][j] == best[i + 1][j + 1] + pair) {
                pairs.push((Some(old[i]), Some(new[j])));
                i += 1;
                j += 1;
            } else if best[i + 1][j] >= best[i][j + 1] {
                pairs.push((Some(old[i]), None));
                i += 1;
            } else {
                pairs.push((None, Some(new[j])));
                j += 1;
            }
        }
        pairs.extend(old[i..].iter().map(|row| (Some(*row), None)));
        pairs.extend(new[j..].iter().map(|row| (None, Some(*row))));
        pairs
    }

    /// One line per row: `#  kind  +disp  width`; with the old rows, `#  kind  old +disp  new +disp  delta` and a
    /// last line that counts each delta, most frequent first.
    pub fn render(old: Option<&[FieldAccessRow]>, new: &[FieldAccessRow]) -> String {
        let Some(old) = old else {
            return new
                .iter()
                .enumerate()
                .map(|(n, row)| format!("{n}  {}  +{:#x}  {}", row.kind.label(), row.disp, row.width))
                .collect::<Vec<_>>()
                .join("\n");
        };
        let side = |row: Option<FieldAccessRow>| row.map_or_else(|| "-".to_string(), |row| format!("+{:#x}", row.disp));
        let mut counts: BTreeMap<i64, usize> = BTreeMap::new();
        let mut lines = Vec::new();
        for (n, (before, after)) in Self::align(old, new).into_iter().enumerate() {
            let kind = before.or(after).map_or("-", |row| row.kind.label());
            let delta = match (before, after) {
                (Some(before), Some(after)) => {
                    let delta = i64::from(after.disp) - i64::from(before.disp);
                    *counts.entry(delta).or_default() += 1;
                    Self::signed_hex(delta)
                }
                _ => "-".to_string(),
            };
            lines.push(format!("{n}  {kind}  {}  {}  {delta}", side(before), side(after)));
        }
        let mut summary: Vec<(i64, usize)> = counts.into_iter().collect();
        summary.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let summary: Vec<String> =
            summary.iter().map(|(delta, count)| format!("{} x{count}", Self::signed_hex(*delta))).collect();
        lines.push(format!("deltas: {}", summary.join(", ")));
        lines.join("\n")
    }

    fn signed_hex(value: i64) -> String {
        if value < 0 { format!("-{:#x}", value.unsigned_abs()) } else { format!("+{value:#x}") }
    }
}
