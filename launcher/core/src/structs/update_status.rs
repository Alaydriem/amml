#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate,
    Available,
    // the installed version is not known (a mod built from source)
    Unknown,
    // no registry lists the mod
    NotListed,
    // installed from a URL; never checked
    Unofficial,
    // the registry could not be reached and nothing is cached
    Offline,
    // the AMML is a local build; `available` names the newest core release for the game, if any
    LocalBuild,
    // no release for the instance's API fits its AMML; `UpdateRow.needs_amml` names the AMML that would
    NeedsAmml,
}
