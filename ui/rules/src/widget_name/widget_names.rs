use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

use super::TypeDeclaration;
use crate::source::SourceFiles;

/// The public type names of a widget folder, so other crates can be checked for a second widget of the same name
/// (rule 4: one widget per Paper sheet).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WidgetNames(BTreeSet<String>);

impl WidgetNames {
    /// Every `pub struct` and `pub enum` name under the folder.
    pub fn from_source(files: &SourceFiles) -> io::Result<Self> {
        let mut names = BTreeSet::new();
        for file in files.rust_files()? {
            let text = fs::read_to_string(&file)?;
            for declaration in text.lines().filter_map(TypeDeclaration::read) {
                if declaration.public {
                    names.insert(declaration.name);
                }
            }
        }
        Ok(Self(names))
    }

    /// The names in amml-ui's `src/widget` folder, found in the sibling `ui/kit` crate of the amml workspace.
    pub fn amml_ui() -> io::Result<Self> {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../kit/src/widget");
        Self::from_source(&SourceFiles::new(folder))
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.contains(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
