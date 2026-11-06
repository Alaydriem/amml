use crate::errors::RegistryError;

/// Where release assets come from. Tests use a folder; the CLI uses GitHub.
pub trait ReleaseSource {
    fn fetch(&self, repo: &str, tag: &str, asset: &str) -> Result<Vec<u8>, RegistryError>;

    /// The URL the launcher downloads the asset from; this is what the registry tree records.
    fn asset_url(&self, repo: &str, tag: &str, asset: &str) -> String {
        format!("https://github.com/{repo}/releases/download/{tag}/{asset}")
    }
}
