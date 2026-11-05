use crate::errors::RegistryError;
use crate::structs::{ModManifest, ReleaseVersion};

/// Reads the fields of a mod `amml.hcl` that the registry needs, including the `listing` block.
pub struct ManifestReader;

impl ManifestReader {
    /// The oldest AMML version a mod may name in `amml_version`. Older AMML builds do not know the field.
    pub const MIN_AMML_FLOOR: &'static str = "26.62.0";

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
        if let Some(got) = &manifest.amml_version {
            Self::check_amml_version(got)?;
        }
        Ok(manifest)
    }

    /// Shape only: the launcher also reads hand-placed and URL mods, so it must not apply the registry floor.
    fn check_amml_version(got: &str) -> Result<(), RegistryError> {
        let bad = |reason: String| RegistryError::MinAmml { got: got.to_string(), reason };
        let shape = "it must have the form MAJOR.MINOR.PATCH, for example 26.62.0".to_string();
        ReleaseVersion::parse(got).map_err(|_| bad(shape.clone()))?;
        // the loader refuses a part with a leading zero, so the registry must refuse it too
        if got.split('.').any(|part| part.len() > 1 && part.starts_with('0')) {
            return Err(bad(shape));
        }
        Ok(())
    }

    /// The registry-only rule: a published `amml_version` must not be older than `MIN_AMML_FLOOR`.
    pub fn check_floor(manifest: &ModManifest) -> Result<(), RegistryError> {
        let Some(got) = &manifest.amml_version else {
            return Ok(());
        };
        let floor = ReleaseVersion::parse(Self::MIN_AMML_FLOOR)?;
        if ReleaseVersion::parse(got)? < floor {
            return Err(RegistryError::MinAmml {
                got: got.clone(),
                reason: format!("it must be {} or newer", Self::MIN_AMML_FLOOR),
            });
        }
        Ok(())
    }
}
