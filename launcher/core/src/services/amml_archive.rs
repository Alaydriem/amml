use super::ZipGuard;
use crate::LauncherError;
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

// An AMML release zip: the loader, the patcher, the recipes, optional solved data (`amml/data`) and the version file.
#[derive(Debug, Clone)]
pub struct AmmlArchive {
    path: PathBuf,
    version: String,
    api: Option<u32>,
}

impl AmmlArchive {
    const VERSION_LIMIT: u64 = 4096;

    pub fn open(path: &Path) -> Result<AmmlArchive, LauncherError> {
        let mut archive = Self::archive(path)?;
        let entries = ZipGuard::entries(&mut archive)?;
        let bad = |reason: &str| ZipGuard::bad(path.to_path_buf(), reason.to_string());
        let mut files = Vec::new();
        for (index, entry) in &entries {
            let is_dir = archive.by_index_raw(*index).map(|file| file.is_dir()).unwrap_or(false);
            if !is_dir {
                files.push((*index, entry.clone()));
            }
        }
        let has = |wanted: &str| files.iter().any(|(_, entry)| *entry == Path::new(wanted));
        if !has("amml.dll") {
            return Err(bad("has no amml.dll"));
        }
        if !has("amml-patch.exe") {
            return Err(bad("has no amml-patch.exe"));
        }
        let recipes = Path::new("amml").join("recipes");
        if !files.iter().any(|(_, entry)| entry.starts_with(&recipes)) {
            return Err(bad("has no recipes"));
        }
        let version_path = Path::new("amml").join("amml-version.txt");
        let Some((index, _)) = files.iter().find(|(_, entry)| *entry == version_path) else {
            return Err(bad("has no amml/amml-version.txt"));
        };
        let bytes = ZipGuard::read_entry(&mut archive, *index, Self::VERSION_LIMIT)?;
        let version = String::from_utf8_lossy(&bytes).trim().to_string();
        if version.is_empty() {
            return Err(bad("has an empty amml/amml-version.txt"));
        }
        // optional: only a local build's zip carries the API of its loader
        let api_path = Path::new("amml").join("amml-api.txt");
        let api = match files.iter().find(|(_, entry)| *entry == api_path) {
            Some((index, _)) => {
                let bytes = ZipGuard::read_entry(&mut archive, *index, Self::VERSION_LIMIT)?;
                String::from_utf8_lossy(&bytes).trim().parse::<u32>().ok()
            }
            None => None,
        };
        Ok(AmmlArchive { path: path.to_path_buf(), version, api })
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    // the API of the loader in the zip, when the zip states one
    pub fn api(&self) -> Option<u32> {
        self.api
    }

    pub fn extract_to(&self, dest: &Path) -> Result<(), LauncherError> {
        let mut archive = Self::archive(&self.path)?;
        let entries = ZipGuard::entries(&mut archive)?;
        ZipGuard::extract(&mut archive, &entries, Path::new(""), dest)
    }

    fn archive(path: &Path) -> Result<ZipArchive<File>, LauncherError> {
        let file = File::open(path)?;
        ZipArchive::new(file).map_err(|error| ZipGuard::bad(path.to_path_buf(), error.to_string()))
    }
}
