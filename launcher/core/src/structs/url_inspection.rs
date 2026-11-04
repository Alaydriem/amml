use crate::{LauncherError, ModId, ModSource, ResolvedMod, VerifiedFile};
use amml_registry::Listing;

// A zip downloaded from a URL and opened, before anything is installed from it.
#[derive(Debug, Clone)]
pub struct UrlInspection {
    pub url: String,
    pub file: VerifiedFile,
    pub id: ModId,
    pub listing: Option<Listing>,
    pub api_version: u32,
    // the registry release this zip is, when some registry lists it
    pub matched: Option<ResolvedMod>,
}

impl UrlInspection {
    // A zip some registry lists installs as that registry's mod. Any other zip is unofficial and needs consent.
    pub fn source(&self, consent: bool) -> Result<ModSource, LauncherError> {
        match &self.matched {
            Some(resolved) => Ok(ModSource::Registry(resolved.registry.base.clone())),
            None if consent => Ok(ModSource::Url(self.url.clone())),
            None => Err(LauncherError::UnofficialNeedsConsent { url: self.url.clone() }),
        }
    }
}
