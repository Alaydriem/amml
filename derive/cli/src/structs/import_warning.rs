use std::path::PathBuf;

/// A mod DLL import that the stable API does not allow, or an import table that could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportWarning {
    pub dll: PathBuf,
    pub function: String,
    pub note: Option<String>,
}
