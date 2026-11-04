use crate::LauncherError;
use crate::LauncherSettings;
use sha2::{Digest, Sha256};

// One registry, named by its base URL (always `https://` and always with a trailing slash).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegistryRef {
    pub base: String,
    // set only for a registry the caller names as the official one without using the official URL
    official: bool,
}

impl RegistryRef {
    pub fn new(base: String) -> RegistryRef {
        RegistryRef { base, official: false }
    }

    // A registry that counts as the official one whatever its URL. The settings file never produces one.
    pub fn official_at(base: String) -> RegistryRef {
        RegistryRef { base, official: true }
    }

    pub fn parse(raw: &str) -> Result<RegistryRef, LauncherError> {
        let trimmed = raw.trim();
        let bad = || LauncherError::BadSettings(format!("{raw} is not an https:// registry URL"));
        let host = trimmed.strip_prefix("https://").ok_or_else(bad)?;
        if host.is_empty()
            || host.starts_with('/')
            || trimmed.chars().any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(bad());
        }
        let base = if trimmed.ends_with('/') { trimmed.to_string() } else { format!("{trimmed}/") };
        Ok(RegistryRef::new(base))
    }

    pub fn url(&self, relative: &str) -> String {
        format!("{}{}", self.base, relative)
    }

    pub fn is_official(&self) -> bool {
        self.official || self.base == LauncherSettings::OFFICIAL_REGISTRY
    }

    // the first 16 hex characters of the SHA-256 of the base URL
    pub fn cache_key(&self) -> String {
        let digest = Sha256::digest(self.base.as_bytes());
        digest.iter().take(8).map(|byte| format!("{byte:02x}")).collect()
    }
}
