use crate::ModSource;

// One mod in the launcher block: the version the launcher installed (if known) and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherModEntry {
    pub version: Option<String>,
    pub source: ModSource,
    // whether the official registry called the mod core when it was installed; `None` when that was not known
    pub core: Option<bool>,
}
