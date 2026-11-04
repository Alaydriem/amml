use crate::InstanceState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewInstanceOutcome {
    pub state: InstanceState,
    // the game version was put into the version store by this call
    pub version_installed: bool,
    // default mods that failed to install: id and reason
    pub skipped_mods: Vec<(String, String)>,
}
