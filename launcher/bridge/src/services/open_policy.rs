use crate::BridgeError;
use std::path::{Path, PathBuf};

// What the window may ask the system to open: folders inside the launcher's own folders, and web pages.
pub struct OpenPolicy;

impl OpenPolicy {
    const SCHEMES: [&'static str; 2] = ["https://", "ms-windows-store://"];

    // The path as it was given, when it is a folder that lies inside one of the roots. `..` and links that lead out are
    // resolved for the check, and a file is refused: opening one would run it.
    pub fn path(roots: &[PathBuf], path: &str) -> Result<PathBuf, BridgeError> {
        let resolved = std::fs::canonicalize(path).map_err(|_| BridgeError::Refused(path.to_string()))?;
        let inside = roots
            .iter()
            .filter_map(|root| std::fs::canonicalize(root).ok())
            .any(|root| Self::contains(&root, &resolved));
        if inside && resolved.is_dir() { Ok(PathBuf::from(path)) } else { Err(BridgeError::Refused(path.to_string())) }
    }

    // The address when it is a web page or a Store page and holds no space or control character.
    pub fn url(url: &str) -> Result<String, BridgeError> {
        let lower = url.to_ascii_lowercase();
        let allowed = Self::SCHEMES.iter().any(|scheme| lower.starts_with(scheme))
            && !url.chars().any(|character| character.is_whitespace() || character.is_control());
        if allowed { Ok(url.to_string()) } else { Err(BridgeError::Refused(url.to_string())) }
    }

    fn contains(root: &Path, path: &Path) -> bool {
        path.starts_with(root)
    }
}
