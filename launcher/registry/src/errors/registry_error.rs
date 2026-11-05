#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("{0} is not in the allow list")]
    NotAllowed(String),
    #[error("repo mismatch: the allow list has {expected}, the release says {got}")]
    RepoMismatch { expected: String, got: String },
    #[error("version {version} is already listed with a different SHA-256")]
    Conflict { version: String },
    #[error("bad version `{0}`")]
    BadVersion(String),
    #[error("bad allow list: {0}")]
    BadAllowList(String),
    #[error("json: {0}")]
    Json(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("release asset not found: {0}")]
    Missing(String),
    #[error("SHA-256 of {asset} does not match SHA256SUMS.txt")]
    HashMismatch { asset: String },
    #[error("network: {0}")]
    Network(String),
    #[error("bad zip: {0}")]
    BadZip(String),
    #[error("manifest field `{field}` is wrong: expected `{expected}`, got `{got}`")]
    ManifestMismatch { field: &'static str, expected: String, got: String },
    #[error("listing field `{field}`: {reason}")]
    Listing { field: &'static str, reason: String },
    #[error("amml_version `{got}`: {reason}")]
    MinAmml { got: String, reason: String },
    #[error("bad registry tree: {0}")]
    BadTree(String),
    #[error("{0}")]
    Usage(String),
}
