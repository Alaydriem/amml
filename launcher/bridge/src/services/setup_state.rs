use crate::SetupView;
use amml_launcher_core::LauncherPaths;
use std::path::Path;

// Whether AMML is on this PC already. The first-run steps are for a PC where it is not.
pub struct SetupState;

impl SetupState {
    // True when the launcher's settings exist, or `instances\` or `versions\` holds a folder that is not hidden.
    // The `launcher\logs\` folder the app makes at start does not count.
    pub fn installed(paths: &LauncherPaths) -> bool {
        paths.settings().exists()
            || Self::holds_a_folder(&paths.gdk().instances())
            || Self::holds_a_folder(&paths.gdk().versions())
    }

    pub fn view(paths: &LauncherPaths) -> SetupView {
        SetupView { needed: !Self::installed(paths) }
    }

    fn holds_a_folder(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        entries.flatten().any(|entry| {
            entry.file_type().is_ok_and(|kind| kind.is_dir()) && !entry.file_name().to_string_lossy().starts_with('.')
        })
    }
}
