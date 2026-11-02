use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::RegistryError;
use crate::structs::{ModDocument, PublishedTree, RegistryRoot};

/// A registry tree on disk: `mods/@<ns>/<name>.json`, `icons/`, and the derived files.
pub struct TreeStore {
    root: PathBuf,
}

impl TreeStore {
    const MODS: &'static str = "mods";
    const ICONS: &'static str = "icons";
    const INDEX: &'static str = "index";
    const ROOT_FILE: &'static str = "registry.json";

    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn load_root(&self) -> Result<Option<RegistryRoot>, RegistryError> {
        match fs::read(self.root.join(Self::ROOT_FILE)) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|error| RegistryError::BadTree(format!("{}: {error}", Self::ROOT_FILE))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn load_document(&self, namespace: &str, name: &str) -> Result<Option<ModDocument>, RegistryError> {
        let path = self.document_path(namespace, name);
        match fs::read(&path) {
            Ok(bytes) => Self::parse_document(&path, &bytes).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Every mod document, in path order.
    pub fn load_documents(&self) -> Result<Vec<ModDocument>, RegistryError> {
        let mut documents = Vec::new();
        let mods = self.root.join(Self::MODS);
        if !mods.is_dir() {
            return Ok(documents);
        }
        for namespace in Self::sorted_entries(&mods)? {
            if !namespace.is_dir() {
                continue;
            }
            for file in Self::sorted_entries(&namespace)? {
                if file.extension().is_some_and(|extension| extension == "json") {
                    documents.push(Self::parse_document(&file, &fs::read(&file)?)?);
                }
            }
        }
        Ok(documents)
    }

    pub fn write_document(&self, document: &ModDocument, bytes: &[u8]) -> Result<(), RegistryError> {
        self.write_if_changed(&self.document_path(&document.namespace, &document.name), bytes)
    }

    pub fn write_icon(&self, namespace: &str, name: &str, bytes: &[u8]) -> Result<(), RegistryError> {
        self.write_if_changed(
            &self.root.join(Self::ICONS).join(format!("@{namespace}")).join(format!("{name}.png")),
            bytes,
        )
    }

    pub fn remove_icon(&self, namespace: &str, name: &str) -> Result<(), RegistryError> {
        let path = self.root.join(Self::ICONS).join(format!("@{namespace}")).join(format!("{name}.png"));
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    /// Replaces the derived files. Index pages that no longer exist are removed.
    pub fn write_tree(&self, tree: &PublishedTree) -> Result<(), RegistryError> {
        let index = self.root.join(Self::INDEX);
        if index.is_dir() {
            fs::remove_dir_all(&index)?;
        }
        for (path, bytes) in &tree.files {
            self.write_if_changed(&self.root.join(path), bytes)?;
        }
        Ok(())
    }

    fn document_path(&self, namespace: &str, name: &str) -> PathBuf {
        self.root.join(Self::MODS).join(format!("@{namespace}")).join(format!("{name}.json"))
    }

    fn parse_document(path: &Path, bytes: &[u8]) -> Result<ModDocument, RegistryError> {
        serde_json::from_slice(bytes).map_err(|error| RegistryError::BadTree(format!("{}: {error}", path.display())))
    }

    fn sorted_entries(dir: &Path) -> Result<Vec<PathBuf>, RegistryError> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(dir)? {
            entries.push(entry?.path());
        }
        entries.sort();
        Ok(entries)
    }

    fn write_if_changed(&self, path: &Path, bytes: &[u8]) -> Result<(), RegistryError> {
        if fs::read(path).is_ok_and(|current| current == bytes) {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
        Ok(())
    }
}
