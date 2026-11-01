use std::path::PathBuf;

use amml_data::EntryId;

/// One validated `amml.hcl`: a mod folder, the DLL it loads and the entries it needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModSpec {
    pub name: String,
    pub dir: PathBuf,
    /// File name of the mod DLL inside `dir`.
    pub entry: String,
    /// The lowest loader API version the mod accepts.
    pub api_version: u32,
    /// Entry ids the mod needs; used for the coverage log line.
    pub entries: Vec<EntryId>,
}

impl ModSpec {
    /// The name of the loader's own log file; no mod may take it.
    pub const RESERVED_NAME: &'static str = "loader";

    /// True for a non-empty name of `A-Z a-z 0-9 _ . -` that does not start with a dot and is not
    /// `loader` in any case. The name is used as a file name for the mod's log, so it can never
    /// hold a path separator.
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && !name.eq_ignore_ascii_case(Self::RESERVED_NAME)
            && !name.starts_with('.')
            && name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
    }

    /// Full path of the mod DLL.
    pub fn library_path(&self) -> PathBuf {
        self.dir.join(&self.entry)
    }
}
