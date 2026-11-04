use crate::GdkError;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GameVersion {
    parts: [u32; 4],
}

impl GameVersion {
    fn split(raw: &str) -> Result<[u32; 4], GdkError> {
        let bad = || GdkError::BadVersion(raw.to_string());
        let mut parts = [0u32; 4];
        let mut pieces = raw.split('.');
        for slot in parts.iter_mut() {
            let piece = pieces.next().ok_or_else(bad)?;
            if piece.is_empty() || !piece.bytes().all(|b| b.is_ascii_digit()) {
                return Err(bad());
            }
            *slot = piece.parse().map_err(|_| bad())?;
        }
        if pieces.next().is_some() {
            return Err(bad());
        }
        Ok(parts)
    }

    pub fn parse(raw: &str) -> Result<GameVersion, GdkError> {
        Ok(GameVersion { parts: Self::split(raw)? })
    }

    // package versions pack minor and patch into the third part as minor * 100 + patch
    pub fn from_package_version(raw: &str) -> Result<GameVersion, GdkError> {
        let packed = Self::split(raw)?;
        Ok(GameVersion { parts: [packed[0], packed[1], packed[2] / 100, packed[2] % 100] })
    }

    // the folder name of a version: the last part is padded to two digits
    pub fn instance_name(&self) -> String {
        format!("{}.{}.{}.{:02}", self.parts[0], self.parts[1], self.parts[2], self.parts[3])
    }
}

impl fmt::Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.parts[0], self.parts[1], self.parts[2], self.parts[3])
    }
}
