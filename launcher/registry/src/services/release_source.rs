use crate::errors::RegistryError;

/// Where release assets come from. Tests use a folder; the CLI uses GitHub.
pub trait ReleaseSource {
    fn fetch(&self, repo: &str, tag: &str, asset: &str) -> Result<Vec<u8>, RegistryError>;
}
