use crate::errors::CliError;
use crate::structs::Image;

/// An address typed on the command line: an RVA or a VA, in hex with an optional `0x` prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    value: u64,
}

impl Address {
    pub fn parse(text: &str) -> Result<Self, CliError> {
        let trimmed = text.trim();
        let digits = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")).unwrap_or(trimmed);
        let value = u64::from_str_radix(digits, 16).map_err(|_| CliError::BadAddress(text.to_string()))?;
        Ok(Self { value })
    }

    /// The RVA in `image`. A value at or above the image base is a VA.
    pub fn rva(&self, image: &Image) -> Result<u32, CliError> {
        let relative = if self.value >= image.image_base() { self.value - image.image_base() } else { self.value };
        let rva = u32::try_from(relative).map_err(|_| CliError::BadAddress(format!("{:#x}", self.value)))?;
        match image.section_of(rva) {
            Some(_) => Ok(rva),
            None => Err(CliError::AddressUnmapped(rva)),
        }
    }
}
