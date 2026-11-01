use crate::errors::StepError;
use crate::services::{FixedFields, PatternScanner};
use crate::structs::{Image, Pattern};

pub struct PatternDrafter<'a> {
    image: &'a Image,
    fixed: FixedFields<'a>,
}

impl<'a> PatternDrafter<'a> {
    pub fn new(image: &'a Image) -> Self {
        Self { image, fixed: FixedFields::new(image) }
    }

    /// A pattern of whole instructions from `rva` that covers at least `len` bytes, with `??` on every byte of
    /// an immediate, displacement or branch field, and its number of matches in the executable sections. The
    /// draft ends at the end of the function range.
    pub fn draft(&self, rva: u32, len: usize) -> Result<(String, usize), StepError> {
        let instructions = self.fixed.instructions(rva, len)?;
        let end = instructions.last().map_or(rva, |last| last.ip + last.len as u32);
        let bytes = self.image.read(rva, (end - rva) as usize).ok_or(StepError::BadInstruction(rva))?;
        let mut tokens: Vec<String> = bytes.iter().map(|byte| format!("{byte:02X}")).collect();
        for (at, _) in self.fixed.fields(rva, len)? {
            if let Some(token) = tokens.get_mut((at - rva) as usize) {
                *token = "??".to_string();
            }
        }
        let text = tokens.join(" ");
        let pattern = Pattern::parse(&text).map_err(|error| StepError::BadPattern(error.to_string()))?;
        Ok((text, PatternScanner::find_all(self.image, &pattern).len()))
    }
}
