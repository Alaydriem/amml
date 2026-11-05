use crate::structs::AmmlState;
use amml_registry::ReleaseVersion;

// How new the AMML of an instance is, for the check against a mod's minimum AMML.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmmlLevel {
    // a core release with a known version
    Release(ReleaseVersion),
    // a local build; it meets every minimum
    Local,
    // no version, or a version that does not parse; it meets no minimum
    Unknown,
}

impl AmmlLevel {
    pub fn of(amml: &AmmlState) -> AmmlLevel {
        if amml.local {
            return AmmlLevel::Local;
        }
        match amml.version.as_deref().map(ReleaseVersion::parse) {
            Some(Ok(version)) => AmmlLevel::Release(version),
            _ => AmmlLevel::Unknown,
        }
    }
}
