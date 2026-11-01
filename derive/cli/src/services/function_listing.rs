use iced_x86::{Decoder, DecoderOptions, FlowControl, Formatter, Instruction, IntelFormatter, OpKind};

use crate::errors::StepError;
use crate::structs::Image;

const MAX_INSTRUCTION_LEN: usize = 15;
const DEFAULT_LIMIT: usize = 400;

pub struct FunctionListing<'a> {
    image: &'a Image,
}

impl<'a> FunctionListing<'a> {
    pub fn new(image: &'a Image) -> Self {
        Self { image }
    }

    /// The primary range of the function that holds `rva`, then every fragment range.
    pub fn ranges(&self, rva: u32) -> Result<String, StepError> {
        let (primary, _) = self.image.functions().containing(rva).ok_or(StepError::NotInListedFunction(rva))?;
        let mut lines = Vec::new();
        for (start, end) in self.image.functions().ranges_of(primary) {
            let line = format!("{} {start:#x}..{end:#x}", if start == primary { "primary" } else { "fragment" });
            if start == primary {
                lines.insert(0, line);
            } else {
                lines.push(line);
            }
        }
        Ok(lines.join("\n"))
    }

    /// The instructions from `rva` to the end of its range, or `count` of them. Without `count` the listing
    /// holds at most 400 instructions.
    pub fn listing(&self, rva: u32, count: Option<usize>) -> Result<String, StepError> {
        let functions = self.image.functions();
        let (primary, end) = functions.containing(rva).ok_or(StepError::NotInListedFunction(rva))?;
        let start = functions
            .ranges_of(primary)
            .into_iter()
            .find(|&(start, range_end)| start <= rva && rva < range_end)
            .map_or(primary, |(start, _)| start);
        let bytes = self.image.read(start, (end - start) as usize).ok_or(StepError::BadInstruction(start))?;
        let mut decoder = Decoder::with_ip(64, bytes, u64::from(start), DecoderOptions::NONE);
        let mut formatter = Self::formatter();
        let mut instruction = Instruction::default();

        let mut lines = vec![if start == primary {
            format!("; function {start:#x}..{end:#x}")
        } else {
            format!("; function {start:#x}..{end:#x} (fragment of {primary:#x})")
        }];
        let limit = count.unwrap_or(DEFAULT_LIMIT);
        let mut listed = 0;
        let mut found = false;
        while decoder.can_decode() && listed < limit {
            decoder.decode_out(&mut instruction);
            let at = instruction.ip() as u32;
            if instruction.is_invalid() {
                return Err(StepError::BadInstruction(at));
            }
            if at < rva {
                continue;
            }
            if at > rva && !found {
                return Err(StepError::NotInstructionStart(rva));
            }
            found = true;
            let offset = (at - start) as usize;
            let raw = &bytes[offset..offset + instruction.len()];
            lines.push(self.line(&mut formatter, &instruction, at, raw));
            listed += 1;
        }
        if !found {
            return Err(StepError::NotInstructionStart(rva));
        }
        Ok(lines.join("\n"))
    }

    pub(crate) fn formatter() -> IntelFormatter {
        let mut formatter = IntelFormatter::new();
        let options = formatter.options_mut();
        options.set_hex_prefix("0x");
        options.set_hex_suffix("");
        options.set_uppercase_hex(false);
        options.set_branch_leading_zeros(false);
        options.set_rip_relative_addresses(true);
        formatter
    }

    fn line(&self, formatter: &mut IntelFormatter, instruction: &Instruction, at: u32, raw: &[u8]) -> String {
        let mut text = String::new();
        formatter.format(instruction, &mut text);
        let target_is_function =
            u32::try_from(instruction.near_branch_target()).is_ok_and(|target| self.image.functions().is_start(target));
        if Self::is_direct_branch(instruction) && target_is_function {
            text.push_str(" ; fn");
        }
        let hex: Vec<String> = raw.iter().map(|byte| format!("{byte:02x}")).collect();
        let width = MAX_INSTRUCTION_LEN * 3 - 1;
        format!("{at:#x}  {:<width$}  {text}", hex.join(" "))
    }

    fn is_direct_branch(instruction: &Instruction) -> bool {
        matches!(
            instruction.flow_control(),
            FlowControl::Call | FlowControl::UnconditionalBranch | FlowControl::ConditionalBranch
        ) && instruction.op0_kind() == OpKind::NearBranch64
    }
}
