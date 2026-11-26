use crate::{BlockTagError, BlockTagValue};

/// Writes the little-endian NBT of a block tag as the game reads it with `NbtIo::read` over a `StringByteInput`: an
/// unnamed root compound with `name` (a string), `states` (a compound of the state values, in the order given) and
/// `version` (an int). Strings are a 16-bit little-endian length and the UTF-8 bytes, with no escape.
pub struct BlockTagWriter;

impl BlockTagWriter {
    const END: u8 = 0;
    const BYTE: u8 = 1;
    const INT: u8 = 3;
    const STRING: u8 = 8;
    const COMPOUND: u8 = 10;

    pub fn new() -> Self {
        Self
    }

    /// The tag of the block type `name` with `states` and `version`. `TooLong` for a name, key or text longer than
    /// 65535 bytes.
    pub fn write(
        &self,
        name: &str,
        states: &[(&str, BlockTagValue<'_>)],
        version: i32,
    ) -> Result<Vec<u8>, BlockTagError> {
        let mut out = Vec::new();
        Self::entry(&mut out, Self::COMPOUND, "")?;
        Self::entry(&mut out, Self::STRING, "name")?;
        Self::text(&mut out, name)?;
        Self::entry(&mut out, Self::COMPOUND, "states")?;
        for (key, value) in states {
            match value {
                BlockTagValue::Int(int) => {
                    Self::entry(&mut out, Self::INT, key)?;
                    out.extend_from_slice(&int.to_le_bytes());
                }
                BlockTagValue::Bool(flag) => {
                    Self::entry(&mut out, Self::BYTE, key)?;
                    out.push(u8::from(*flag));
                }
                BlockTagValue::Text(text) => {
                    Self::entry(&mut out, Self::STRING, key)?;
                    Self::text(&mut out, text)?;
                }
            }
        }
        out.push(Self::END);
        Self::entry(&mut out, Self::INT, "version")?;
        out.extend_from_slice(&version.to_le_bytes());
        out.push(Self::END);
        Ok(out)
    }

    fn entry(out: &mut Vec<u8>, kind: u8, name: &str) -> Result<(), BlockTagError> {
        out.push(kind);
        Self::text(out, name)
    }

    fn text(out: &mut Vec<u8>, text: &str) -> Result<(), BlockTagError> {
        let len = u16::try_from(text.len()).map_err(|_| BlockTagError::TooLong { len: text.len() })?;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(text.as_bytes());
        Ok(())
    }
}

impl Default for BlockTagWriter {
    fn default() -> Self {
        Self::new()
    }
}
