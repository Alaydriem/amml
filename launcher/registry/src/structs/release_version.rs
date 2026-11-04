use crate::errors::RegistryError;

/// A `MAJOR.MINOR.PATCH` version of three decimal numbers; ordered by number, not by text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReleaseVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl ReleaseVersion {
    pub fn parse(raw: &str) -> Result<Self, RegistryError> {
        let bad = || RegistryError::BadVersion(raw.to_string());
        let mut parts = raw.split('.');
        let mut next = || -> Result<u64, RegistryError> {
            let part = parts.next().ok_or_else(bad)?;
            if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(bad());
            }
            part.parse::<u64>().map_err(|_| bad())
        };
        let major = next()?;
        let minor = next()?;
        let patch = next()?;
        if parts.next().is_some() {
            return Err(bad());
        }
        Ok(Self { major, minor, patch })
    }
}
