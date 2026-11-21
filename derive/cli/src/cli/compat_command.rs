use std::fs;
use std::path::{Path, PathBuf};

use amml_data::{BuildDataStore, BuildIdReader};

use super::CompatArgs;
use crate::errors::CliError;
use crate::services::CompatChecker;
use crate::structs::{CompatReport, ModUses};

pub struct CompatCommand;

impl CompatCommand {
    /// Reads the build data of `--exe` from `--data`, and the `uses` block and DLL of each mod under `--mods`.
    pub fn run(args: &CompatArgs) -> Result<CompatReport, CliError> {
        let build = BuildIdReader::from_path(&args.exe)?;
        let data = BuildDataStore::read(&args.data.join(format!("{}.hcl", build.file_stem())))?;
        let mut mods = Vec::new();
        for dir in Self::mod_dirs(&args.mods)? {
            mods.push(ModUses::read(&dir)?);
        }
        let dlls: Vec<PathBuf> = mods.iter().filter_map(|uses| uses.dll.clone()).collect();
        Ok(CompatChecker::check(&data, &mods, &dlls))
    }

    fn sorted_children(path: &Path) -> Result<Vec<PathBuf>, CliError> {
        let items = fs::read_dir(path)
            .map_err(|error| CliError::Manifest { path: path.to_path_buf(), message: error.to_string() })?;
        let mut paths: Vec<PathBuf> = items.filter_map(|item| item.ok().map(|item| item.path())).collect();
        paths.sort();
        Ok(paths)
    }

    // `<root>/@<namespace>/<name>` folders that hold a manifest, in sorted path order.
    fn mod_dirs(root: &Path) -> Result<Vec<PathBuf>, CliError> {
        let mut dirs = Vec::new();
        for namespace in Self::sorted_children(root)? {
            let is_namespace = namespace.file_name().is_some_and(|name| name.to_string_lossy().starts_with('@'));
            if !namespace.is_dir() || !is_namespace {
                continue;
            }
            dirs.extend(
                Self::sorted_children(&namespace)?.into_iter().filter(|dir| dir.join(ModUses::MANIFEST).is_file()),
            );
        }
        Ok(dirs)
    }
}
