use super::{ModSpec, SkippedMod};

/// The result of reading a mods root: accepted mods in load order and the folders that were skipped.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModScan {
    pub mods: Vec<ModSpec>,
    pub skipped: Vec<SkippedMod>,
}
