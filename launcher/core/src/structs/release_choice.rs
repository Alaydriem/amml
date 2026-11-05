use crate::structs::NeedsAmml;
use amml_registry::ModRelease;

// The newest mod release that fits the AMML of an instance.
// `blocked` is set when the newest release for the API needs a newer AMML.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseChoice {
    pub fitting: Option<ModRelease>,
    pub blocked: Option<NeedsAmml>,
}
