use std::path::PathBuf;

/// A mod folder the registry did not accept, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkippedMod {
    pub dir: PathBuf,
    pub reason: String,
}
