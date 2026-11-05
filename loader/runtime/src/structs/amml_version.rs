use std::fmt;

/// A release version of AMML, `major.minor.patch`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AmmlVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl AmmlVersion {
    /// Reads exactly `X.Y.Z`, with no leading zeros. A `+suffix` is left out.
    pub fn parse(text: &str) -> Option<AmmlVersion> {
        let core = text.split_once('+').map_or(text, |(core, _)| core);
        let mut parts = core.split('.');
        let major = Self::number(parts.next()?)?;
        let minor = Self::number(parts.next()?)?;
        let patch = Self::number(parts.next()?)?;
        if parts.next().is_some() {
            return None;
        }
        Some(AmmlVersion { major, minor, patch })
    }

    fn number(part: &str) -> Option<u32> {
        let plain = !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
        if !plain || (part.len() > 1 && part.starts_with('0')) {
            return None;
        }
        part.parse().ok()
    }
}

impl fmt::Display for AmmlVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
