use crate::errors::RegistryError;
use crate::structs::ModManifest;

/// Reads the fields of a mod `amml.hcl` that the registry needs, including the `listing` block.
pub struct ManifestReader;

impl ManifestReader {
    pub fn parse(text: &str) -> Result<ModManifest, RegistryError> {
        let body = hcl::parse(text)
            .map_err(|error| RegistryError::Listing { field: "amml.hcl", reason: format!("not valid HCL: {error}") })?;
        let has_listing = body.blocks().any(|block| block.identifier() == "listing");
        let manifest = hcl::from_str::<ModManifest>(text).map_err(|error| RegistryError::Listing {
            field: if has_listing { "listing" } else { "amml.hcl" },
            reason: error.to_string(),
        })?;
        if manifest.api_version == 0 {
            return Err(RegistryError::ManifestMismatch {
                field: "api_version",
                expected: "1 or more".to_string(),
                got: "0".to_string(),
            });
        }
        Ok(manifest)
    }
}
