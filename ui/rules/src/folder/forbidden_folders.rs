use std::path::PathBuf;

/// Module names an app crate must not have (rule 5: app crates have no `widgets/` or `theme/` module).
pub struct ForbiddenFolders {
    root: PathBuf,
    names: Vec<&'static str>,
}

impl ForbiddenFolders {
    pub fn new(root: impl Into<PathBuf>, names: &[&'static str]) -> Self {
        Self { root: root.into(), names: names.to_vec() }
    }

    /// The forbidden modules that exist under the root, as a folder or as a `<name>.rs` file.
    pub fn present(&self) -> Vec<PathBuf> {
        let mut present = Vec::new();
        for name in &self.names {
            let folder = self.root.join(name);
            if folder.is_dir() {
                present.push(folder);
            }
            let file = self.root.join(format!("{name}.rs"));
            if file.is_file() {
                present.push(file);
            }
        }
        present
    }
}
