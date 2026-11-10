use std::fmt;

/// A 64-bit hash of everything that decides one entry's derived value. It prints as 16 lowercase hex digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fingerprint(pub u64);

impl Fingerprint {
    /// Reads exactly 16 hex digits; anything else gives `None`.
    pub fn parse(text: &str) -> Option<Fingerprint> {
        if text.len() != 16 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        u64::from_str_radix(text, 16).ok().map(Fingerprint)
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}
