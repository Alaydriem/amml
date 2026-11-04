// A file to download. The hash and size are `None` only for an unofficial URL install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub url: String,
    pub sha256: Option<String>,
    pub size: Option<u64>,
}
