use std::fs;
use std::path::{Path, PathBuf};

use amml_data::EntryId;
use hcl::{Body, Expression};

use crate::structs::{ModRegistryError, ModScan, ModSpec, SkippedMod};

/// Reads the mod folders under a root: `<root>/<folder>/amml.hcl`.
///
/// Folders are read in sorted name order. A folder without a manifest is not a mod and is ignored;
/// a folder whose manifest is invalid is skipped and reported, and the others still load. At most
/// `limit` valid mods are accepted; later ones are skipped.
pub struct ModRegistry {
    limit: usize,
}

impl ModRegistry {
    /// File name of a mod manifest.
    pub const MANIFEST: &'static str = "amml.hcl";
    /// Most mods a host accepts.
    pub const MAX_MODS: usize = 64;

    pub fn new(limit: usize) -> Self {
        Self { limit }
    }

    /// Reads every mod folder under `root`. A missing root is an empty scan.
    pub fn scan(&self, root: &Path) -> Result<ModScan, ModRegistryError> {
        let io = |source| ModRegistryError::Io { path: root.to_path_buf(), source };
        let mut scan = ModScan::default();
        if !root.exists() {
            return Ok(scan);
        }
        let mut folders: Vec<PathBuf> = Vec::new();
        for item in fs::read_dir(root).map_err(io)? {
            let path = item.map_err(io)?.path();
            if path.is_dir() && path.join(Self::MANIFEST).is_file() {
                folders.push(path);
            }
        }
        folders.sort();
        for dir in folders {
            match Self::read(&dir) {
                Ok(spec) if scan.mods.len() >= self.limit => {
                    let reason = format!("the limit of {} mods is reached, `{}` is not loaded", self.limit, spec.name);
                    scan.skipped.push(SkippedMod { dir, reason });
                }
                Ok(spec) if scan.mods.iter().any(|other| other.name.eq_ignore_ascii_case(&spec.name)) => {
                    let reason = format!("another folder already holds a mod named `{}`", spec.name);
                    scan.skipped.push(SkippedMod { dir, reason });
                }
                Ok(spec) => scan.mods.push(spec),
                Err(reason) => scan.skipped.push(SkippedMod { dir, reason }),
            }
        }
        Ok(scan)
    }

    fn read(dir: &Path) -> Result<ModSpec, String> {
        let path = dir.join(Self::MANIFEST);
        let text = fs::read_to_string(&path).map_err(|error| format!("cannot read {}: {error}", Self::MANIFEST))?;
        let body = hcl::parse(&text).map_err(|error| format!("invalid {}: {error}", Self::MANIFEST))?;
        Self::spec(dir, &body)
    }

    fn spec(dir: &Path, body: &Body) -> Result<ModSpec, String> {
        if body.blocks().next().is_some() {
            return Err("the manifest takes attributes only, not blocks".to_string());
        }
        let mut name = None;
        let mut entry = None;
        let mut api_version = None;
        let mut entries = Vec::new();
        for attribute in body.attributes() {
            let key = attribute.key();
            match (key, attribute.expr()) {
                ("name", Expression::String(text)) => name = Some(text.clone()),
                ("entry", Expression::String(text)) => entry = Some(text.clone()),
                ("api_version", Expression::Number(number)) => {
                    let version = number.as_u64().and_then(|value| u32::try_from(value).ok());
                    api_version = Some(version.ok_or("`api_version` must be a whole number")?);
                }
                ("entries", Expression::Array(items)) => {
                    for item in items {
                        let Expression::String(id) = item else {
                            return Err("`entries` holds strings only".to_string());
                        };
                        if !EntryId::is_valid_text(id) {
                            return Err(format!("`{id}` is not a valid entry id"));
                        }
                        entries.push(EntryId(id.clone()));
                    }
                }
                ("name" | "entry" | "api_version" | "entries", _) => {
                    return Err(format!("`{key}` has the wrong type"));
                }
                _ => return Err(format!("unknown attribute `{key}`")),
            }
        }
        let name = name.ok_or("`name` is missing")?;
        let entry = entry.ok_or("`entry` is missing")?;
        let api_version = api_version.ok_or("`api_version` is missing")?;
        if !ModSpec::is_valid_name(&name) {
            return Err(format!("`{name}` is not a valid mod name"));
        }
        if !Self::is_dll_file_name(&entry) {
            return Err(format!("`{entry}` is not a DLL file name inside the mod folder"));
        }
        if api_version == 0 {
            return Err("`api_version` starts at 1".to_string());
        }
        entries.sort();
        entries.dedup();
        Ok(ModSpec { name, dir: dir.to_path_buf(), entry, api_version, entries })
    }

    fn is_dll_file_name(entry: &str) -> bool {
        let plain = !entry.contains(['/', '\\', ':']) && entry != ".." && entry != ".";
        plain && entry.to_ascii_lowercase().ends_with(".dll")
    }
}
