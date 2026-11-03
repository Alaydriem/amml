use super::ZipGuard;
use crate::{LauncherError, ModId};
use amml_registry::{ManifestReader, ModManifest};
use std::collections::BTreeSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

// A mod release zip: one folder `@namespace/` that holds one folder `name/` with the mod's `amml.hcl`.
#[derive(Debug, Clone)]
pub struct ModArchive {
    path: PathBuf,
    pub id: ModId,
    pub manifest: ModManifest,
}

impl ModArchive {
    const MANIFEST_LIMIT: u64 = 1024 * 1024;

    pub fn open(path: &Path) -> Result<ModArchive, LauncherError> {
        let mut archive = Self::archive(path)?;
        let entries = ZipGuard::entries(&mut archive)?;
        let bad = |reason: &str| ZipGuard::bad(path.to_path_buf(), reason.to_string());

        let mut namespaces = BTreeSet::new();
        let mut names = BTreeSet::new();
        for (index, entry) in &entries {
            let parts: Vec<String> = entry.iter().map(|part| part.to_string_lossy().to_string()).collect();
            let is_dir = archive.by_index_raw(*index).map(|file| file.is_dir()).unwrap_or(false);
            namespaces.insert(parts[0].clone());
            match parts.len() {
                1 if !is_dir => return Err(bad("has a file outside the mod folder")),
                2 if !is_dir => return Err(bad("has a file outside the mod folder")),
                1 => {}
                _ => {
                    names.insert(parts[1].clone());
                }
            }
        }
        if namespaces.len() != 1 || !namespaces.iter().all(|namespace| namespace.starts_with('@')) {
            return Err(bad("must hold exactly one @namespace folder"));
        }
        if names.len() != 1 {
            return Err(bad("must hold exactly one mod folder"));
        }
        let namespace = namespaces.into_iter().next().unwrap_or_default();
        let name = names.into_iter().next().unwrap_or_default();
        let id = ModId::from_parts(namespace.trim_start_matches('@'), &name)?;

        let manifest_path = Path::new(&namespace).join(&name).join("amml.hcl");
        let Some((index, _)) = entries.iter().find(|(_, entry)| *entry == manifest_path) else {
            return Err(bad("has no amml.hcl in the mod folder"));
        };
        let bytes = ZipGuard::read_entry(&mut archive, *index, Self::MANIFEST_LIMIT)?;
        let text = String::from_utf8(bytes).map_err(|_| bad("has an amml.hcl that is not UTF-8"))?;
        let manifest = ManifestReader::parse(&text)?;
        if manifest.namespace != id.namespace || manifest.name != id.name {
            return Err(bad(&format!(
                "has an amml.hcl for @{}/{}, not for the folder {}",
                manifest.namespace,
                manifest.name,
                id.as_str()
            )));
        }
        Ok(ModArchive { path: path.to_path_buf(), id, manifest })
    }

    pub fn api_version(&self) -> u32 {
        self.manifest.api_version
    }

    // writes the contents of `@namespace/name/` into `dest`
    pub fn extract_to(&self, dest: &Path) -> Result<(), LauncherError> {
        let mut archive = Self::archive(&self.path)?;
        let entries = ZipGuard::entries(&mut archive)?;
        let prefix = Path::new(&format!("@{}", self.id.namespace)).join(&self.id.name);
        ZipGuard::extract(&mut archive, &entries, &prefix, dest)
    }

    fn archive(path: &Path) -> Result<ZipArchive<File>, LauncherError> {
        let file = File::open(path)?;
        ZipArchive::new(file).map_err(|error| ZipGuard::bad(path.to_path_buf(), error.to_string()))
    }
}
