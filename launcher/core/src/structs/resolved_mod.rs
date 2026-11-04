use crate::RegistryRef;
use amml_registry::{ModDocument, ModRelease};

// A mod release picked for an install: which registry it came from and whether the document was a cached copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMod {
    pub registry: RegistryRef,
    pub document: ModDocument,
    pub release: ModRelease,
    // the registry could not be reached; the document is the saved copy
    pub stale: bool,
    // the document says `core` and the registry is the official one
    pub core: bool,
}
