use std::path::PathBuf;

use crate::structs::EntryId;

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("io error on {}: {source}", path.display())]
    Io { path: PathBuf, source: std::io::Error },
    #[error("{}{}: {message}", path.display(), Self::entry_part(.entry))]
    Parse { path: PathBuf, entry: Option<String>, message: String },
    #[error("{}duplicate entry id {id}", Self::file_prefix(.path))]
    DuplicateId { id: EntryId, path: Option<PathBuf> },
    #[error("{}entry {entry} refers to unknown entry {target}", Self::file_prefix(.path))]
    UnknownRef { entry: EntryId, target: EntryId, path: Option<PathBuf> },
    #[error("no entry named {0}")]
    UnknownEntry(EntryId),
    #[error("dependency cycle among {0:?}")]
    Cycle(Vec<EntryId>),
    #[error("{}entry {entry} recipe {recipe} must start with a source step", Self::file_prefix(.path))]
    BadFirstStep { entry: EntryId, recipe: usize, path: Option<PathBuf> },
    #[error("{}entry {id} has no recipes", Self::file_prefix(.path))]
    EmptyRecipes { id: EntryId, path: Option<PathBuf> },
    #[error("{}entry {entry}: {message}", Self::file_prefix(.path))]
    Invalid { entry: EntryId, message: String, path: Option<PathBuf> },
    #[error("bad pattern token `{token}`")]
    BadPatternToken { token: String },
    #[error("{} is not a valid 64-bit PE image: {message}", path.display())]
    NotPe { path: PathBuf, message: String },
}

impl DataError {
    // "<path>: " when the source file is known, otherwise nothing
    fn file_prefix(path: &Option<PathBuf>) -> String {
        path.as_ref().map(|path| format!("{}: ", path.display())).unwrap_or_default()
    }

    // " (entry <id>)" when the error belongs to one entry, otherwise nothing
    fn entry_part(entry: &Option<String>) -> String {
        entry.as_ref().map(|entry| format!(" (entry {entry})")).unwrap_or_default()
    }
}
