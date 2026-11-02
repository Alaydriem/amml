use sha2::{Digest, Sha256};

/// Lower-case hex SHA-256 text.
pub struct HashText;

impl HashText {
    pub fn sha256(bytes: &[u8]) -> String {
        let digest = Sha256::digest(bytes);
        let mut text = String::with_capacity(64);
        for byte in digest {
            text.push(Self::nibble(byte >> 4));
            text.push(Self::nibble(byte & 0x0f));
        }
        text
    }

    fn nibble(value: u8) -> char {
        char::from_digit(u32::from(value), 16).unwrap_or('0')
    }
}
