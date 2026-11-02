use std::path::PathBuf;

use amml_abi::ModId;
use amml_data::EntryId;

/// One validated `amml.hcl`: a mod folder, the DLL it loads and the entries it needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModSpec {
    pub namespace: String,
    pub name: String,
    pub dir: PathBuf,
    /// File name of the mod DLL inside `dir`.
    pub entry: String,
    /// The lowest loader API version the mod accepts.
    pub api_version: u32,
    /// The release version the mod publishes, when its manifest carries one.
    pub version: Option<String>,
    /// Entry ids the mod needs; used for the coverage log line.
    pub entries: Vec<EntryId>,
}

impl ModSpec {
    /// The mod id, `@<namespace>/<name>`.
    pub fn id(&self) -> String {
        ModId::format(&self.namespace, &self.name)
    }

    /// Full path of the mod DLL.
    pub fn library_path(&self) -> PathBuf {
        self.dir.join(&self.entry)
    }
}
