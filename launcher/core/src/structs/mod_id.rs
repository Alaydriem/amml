use crate::LauncherError;
use std::path::{Path, PathBuf};

// The id of a mod: `@namespace/name`, each part `^[a-z0-9][a-z0-9_-]{0,63}$`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModId {
    pub namespace: String,
    pub name: String,
}

impl ModId {
    const PART_LIMIT: usize = 64;

    pub fn parse(raw: &str) -> Result<ModId, LauncherError> {
        let bad = || LauncherError::BadModId(raw.to_string());
        let rest = raw.strip_prefix('@').ok_or_else(bad)?;
        let (namespace, name) = rest.split_once('/').ok_or_else(bad)?;
        if !Self::valid_part(namespace) || !Self::valid_part(name) {
            return Err(bad());
        }
        Ok(ModId { namespace: namespace.to_string(), name: name.to_string() })
    }

    pub fn from_parts(namespace: &str, name: &str) -> Result<ModId, LauncherError> {
        ModId::parse(&format!("@{namespace}/{name}"))
    }

    pub fn as_str(&self) -> String {
        format!("@{}/{}", self.namespace, self.name)
    }

    // the path of the mod document below the registry base
    pub fn document_path(&self) -> String {
        format!("mods/@{}/{}.json", self.namespace, self.name)
    }

    // `<mods_dir>\@namespace\name`
    pub fn folder(&self, mods_dir: &Path) -> PathBuf {
        mods_dir.join(format!("@{}", self.namespace)).join(&self.name)
    }

    fn valid_part(part: &str) -> bool {
        let mut bytes = part.bytes();
        let Some(first) = bytes.next() else {
            return false;
        };
        part.len() <= Self::PART_LIMIT
            && (first.is_ascii_lowercase() || first.is_ascii_digit())
            && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-')
    }
}
