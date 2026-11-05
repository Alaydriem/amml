use std::fs;
use std::path::{Path, PathBuf};

use amml_abi::ModId;
use amml_data::EntryId;
use hcl::{Body, Expression};

use crate::structs::{AmmlVersion, ModRegistryError, ModScan, ModSpec, SkippedMod};

/// Reads the mod folders under a root: `<root>/@<namespace>/<name>/amml.hcl`.
///
/// Folders are read in sorted path order. A folder without a manifest is not a mod and is ignored; a mod folder directly under the root is skipped with the place to move it to;
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
        let io = |path: &Path| {
            let path = path.to_path_buf();
            move |source| ModRegistryError::Io { path, source }
        };
        let mut scan = ModScan::default();
        if !root.exists() {
            return Ok(scan);
        }
        let mut folders: Vec<(PathBuf, bool)> = Vec::new();
        for item in fs::read_dir(root).map_err(io(root))? {
            let path = item.map_err(io(root))?.path();
            if !path.is_dir() {
                continue;
            }
            let is_namespace = path.file_name().is_some_and(|name| name.to_string_lossy().starts_with('@'));
            if is_namespace {
                for inner in fs::read_dir(&path).map_err(io(&path))? {
                    let dir = inner.map_err(io(&path))?.path();
                    if dir.is_dir() && dir.join(Self::MANIFEST).is_file() {
                        folders.push((dir, true));
                    }
                }
            } else if path.join(Self::MANIFEST).is_file() {
                folders.push((path, false));
            }
        }
        folders.sort();
        for (dir, nested) in folders {
            let read = if nested { Self::read(&dir) } else { Err(Self::flat_reason(&dir)) };
            match read {
                Ok(spec) if scan.mods.len() >= self.limit => {
                    let reason = format!("the limit of {} mods is reached, `{}` is not loaded", self.limit, spec.id());
                    scan.skipped.push(SkippedMod { dir, reason });
                }
                Ok(spec) if scan.mods.iter().any(|other| other.id().eq_ignore_ascii_case(&spec.id())) => {
                    let reason = format!("another folder already holds a mod with the id `{}`", spec.id());
                    scan.skipped.push(SkippedMod { dir, reason });
                }
                Ok(spec) => scan.mods.push(spec),
                Err(reason) => scan.skipped.push(SkippedMod { dir, reason }),
            }
        }
        Ok(scan)
    }

    fn flat_reason(dir: &Path) -> String {
        let pair = fs::read_to_string(dir.join(Self::MANIFEST)).ok().and_then(|text| hcl::parse(&text).ok()).and_then(
            |body| {
                let text = |key: &str| {
                    body.attributes().find(|attribute| attribute.key() == key).and_then(|attribute| {
                        match attribute.expr() {
                            Expression::String(text) => Some(text.clone()),
                            _ => None,
                        }
                    })
                };
                Some((text("namespace")?, text("name")?))
            },
        );
        match pair {
            Some((namespace, name)) => {
                format!(
                    "the folder is not inside a namespace folder; move it to `{}/`",
                    ModId::format(&namespace, &name)
                )
            }
            None => "the folder is not inside a namespace folder; move it into `@<namespace>/<name>/`".to_string(),
        }
    }

    fn read(dir: &Path) -> Result<ModSpec, String> {
        let path = dir.join(Self::MANIFEST);
        let text = fs::read_to_string(&path).map_err(|error| format!("cannot read {}: {error}", Self::MANIFEST))?;
        let body = hcl::parse(&text).map_err(|error| format!("invalid {}: {error}", Self::MANIFEST))?;
        Self::spec(dir, &body)
    }

    fn spec(dir: &Path, body: &Body) -> Result<ModSpec, String> {
        let mut listing_seen = false;
        for block in body.blocks() {
            if block.identifier() != "listing" {
                return Err(format!("unknown block `{}`", block.identifier()));
            }
            if !block.labels().is_empty() {
                return Err("`listing` takes no labels".to_string());
            }
            if listing_seen {
                return Err("only one `listing` block is allowed".to_string());
            }
            listing_seen = true;
        }
        let mut namespace = None;
        let mut name = None;
        let mut entry = None;
        let mut api_version = None;
        let mut version = None;
        let mut amml_version = None;
        let mut entries = Vec::new();
        for attribute in body.attributes() {
            let key = attribute.key();
            match (key, attribute.expr()) {
                ("namespace", Expression::String(text)) => namespace = Some(text.clone()),
                ("name", Expression::String(text)) => name = Some(text.clone()),
                ("entry", Expression::String(text)) => entry = Some(text.clone()),
                ("version", Expression::String(text)) => version = Some(text.clone()),
                ("amml_version", Expression::String(text)) => {
                    let parsed = AmmlVersion::parse(text);
                    amml_version = Some(parsed.ok_or("`amml_version` must look like 26.62.0")?);
                }
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
                ("namespace" | "name" | "entry" | "api_version" | "entries" | "version" | "amml_version", _) => {
                    return Err(format!("`{key}` has the wrong type"));
                }
                _ => return Err(format!("unknown attribute `{key}`")),
            }
        }
        let namespace = namespace.ok_or("`namespace` is missing")?;
        let name = name.ok_or("`name` is missing")?;
        let entry = entry.ok_or("`entry` is missing")?;
        let api_version = api_version.ok_or("`api_version` is missing")?;
        if !ModId::is_valid_segment(&namespace) {
            return Err(format!("`{namespace}` is not a valid mod namespace"));
        }
        if !ModId::is_valid_segment(&name) {
            return Err(format!("`{name}` is not a valid mod name"));
        }
        if ModId::is_reserved(&namespace, &name) {
            return Err(format!("`{}` is reserved for the loader", ModId::format(&namespace, &name)));
        }
        let folder = Self::folder_pair(dir);
        if !folder
            .as_ref()
            .is_some_and(|(ns, nm)| ns.eq_ignore_ascii_case(&namespace) && nm.eq_ignore_ascii_case(&name))
        {
            let found = folder.map_or_else(|| "an unknown folder".to_string(), |(ns, nm)| format!("@{ns}/{nm}"));
            return Err(format!(
                "the folder `{found}` does not match the manifest id `{}`",
                ModId::format(&namespace, &name)
            ));
        }
        if !Self::is_dll_file_name(&entry) {
            return Err(format!("`{entry}` is not a DLL file name inside the mod folder"));
        }
        if api_version == 0 {
            return Err("`api_version` starts at 1".to_string());
        }
        entries.sort();
        entries.dedup();
        Ok(ModSpec { namespace, name, dir: dir.to_path_buf(), entry, api_version, version, amml_version, entries })
    }

    /// The `(namespace, name)` of a `@<namespace>/<name>` folder, without the `@`.
    fn folder_pair(dir: &Path) -> Option<(String, String)> {
        let name = dir.file_name()?.to_string_lossy().into_owned();
        let namespace = dir.parent()?.file_name()?.to_string_lossy().strip_prefix('@')?.to_string();
        Some((namespace, name))
    }

    fn is_dll_file_name(entry: &str) -> bool {
        let plain = !entry.contains(['/', '\\', ':']) && entry != ".." && entry != ".";
        plain && entry.to_ascii_lowercase().ends_with(".dll")
    }
}
