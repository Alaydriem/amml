use crate::LauncherError;
use std::path::Path;

// Writes a patched copy of the game executable.
pub trait Patcher {
    fn patch(&self, patcher_exe: &Path, input: &Path, output: &Path) -> Result<(), LauncherError>;
}
