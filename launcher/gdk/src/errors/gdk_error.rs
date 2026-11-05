#[derive(Debug, thiserror::Error)]
pub enum GdkError {
    #[error("not a game version: {0}")]
    BadVersion(String),
    #[error("the version catalogue is missing its {0} section")]
    BadCatalogue(&'static str),
    #[error("the catalogue has no entry for {0}")]
    UnknownVersion(String),
    #[error("download failed: {0}")]
    Download(String),
    #[error("Windows call {call} failed: {text}")]
    Windows { call: &'static str, text: String },
    #[error("package {action} failed with 0x{hresult:08X}: {text}")]
    Deploy { action: &'static str, hresult: i32, text: String },
    #[error("{0} is the package the launcher restores and is never removed")]
    Protected(String),
    #[error("copying the game executable ({method}) failed: {detail}")]
    ExeCopyFailed { method: &'static str, detail: String },
    #[error("version {0} is already in the version store")]
    VersionExists(String),
    #[error("the copy is incomplete: {copied} of {expected} files")]
    IncompleteCopy { expected: u64, copied: u64 },
    #[error("{} is not a complete game folder", .0.display())]
    NotAGame(std::path::PathBuf),
    #[error("the Store install is not ready: {0:?}")]
    NotReady(Box<crate::structs::StoreState>),
    #[error("the registered version {0} is not in the catalogue, so it could not be restored")]
    CurrentNotInCatalogue(String),
    #[error("{original} was not restored ({cause}); run Add-AppxPackage \"{}\" to put it back", .package.display())]
    RestoreFailed { original: String, package: std::path::PathBuf, cause: String },
    #[error("another install is running")]
    Busy,
    #[error("the install journal is unreadable: {0}")]
    BadJournal(String),
    #[error("expected {expected} to be registered, found {found}")]
    UnexpectedRegistration { expected: String, found: String },
    #[error("{0} is not a usable instance name")]
    BadInstanceName(String),
    #[error("the instance {0} exists already")]
    InstanceExists(String),
    #[error("version.json is not usable: {0}")]
    BadVersionJson(String),
    #[error("the download was cancelled")]
    Cancelled,
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
}
