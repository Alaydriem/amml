use amml_gdk::GdkError;
use amml_registry::RegistryError;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error(transparent)]
    Gdk(#[from] GdkError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{url} answered: {text}")]
    Http { url: String, text: String },
    #[error("{url} is larger than the limit of {limit} bytes")]
    TooLarge { url: String, limit: u64 },
    #[error("{url} has schema {found}, the launcher reads schema 2")]
    BadSchema { url: String, found: u32 },
    #[error("{url} cannot be reached")]
    Offline { url: String },
    #[error("bad launcher settings: {0}")]
    BadSettings(String),
    #[error("bad mod id or path: {0}")]
    BadModId(String),
    #[error("bad archive {}: {reason}", .file.display())]
    BadArchive { file: PathBuf, reason: String },
    #[error("checksum mismatch for {file}: expected {expected}, found {found}")]
    ChecksumMismatch { file: String, expected: String, found: String },
    #[error("no AMML release supports the game version {game}")]
    NotSupported { game: String },
    #[error("unknown mod {0}")]
    UnknownMod(String),
    #[error("mod {id} is for API {mod_api}, the instance AMML is API {amml_api}")]
    ApiMismatch { id: String, mod_api: u32, amml_api: u32 },
    #[error("{url} is not in any registry; installing it needs explicit consent")]
    UnofficialNeedsConsent { url: String },
    #[error("{0} is a core mod and cannot be turned off or removed")]
    CoreMod(String),
    #[error("the game is running from the instance {0}")]
    InstanceRunning(String),
    #[error("the confirmation does not match the instance name {name}")]
    ConfirmMismatch { name: String },
    #[error("{} is not an instance", .0.display())]
    NotAnInstance(PathBuf),
    #[error("{} is not empty", .0.display())]
    TargetNotEmpty(PathBuf),
    #[error("the patcher failed (exit {code:?}): {text}")]
    PatchFailed { code: Option<i32>, text: String },
    #[error("bad version.json: {0}")]
    BadVersionJson(String),
    #[error("bad launch setting: {0}")]
    BadLaunchSetting(String),
    #[error("the installer ended with exit code {code:?}")]
    InstallerFailed { code: Option<i32> },
    #[error("the AMML API of the instance {0} is not known")]
    UnknownApi(String),
}
