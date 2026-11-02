use super::{Listing, ModRelease};

/// A mod release that passed every check, with what the registry copies from its zip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedMod {
    pub release: ModRelease,
    pub listing: Listing,
    // PNG bytes of the listing icon, when the listing names one
    pub icon: Option<Vec<u8>>,
}
