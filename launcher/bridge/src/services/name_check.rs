use crate::NameProblemView;
use amml_launcher_core::LauncherPaths;

// Checks a new instance name before anything is downloaded.
pub struct NameCheck;

impl NameCheck {
    pub fn problem(name: &str, taken: &[String], paths: &LauncherPaths) -> Option<NameProblemView> {
        if name.trim().is_empty() {
            return Some(NameProblemView::Empty);
        }
        if paths.checked_instance(name).is_err() {
            return Some(NameProblemView::Bad);
        }
        // Windows folder names are not case sensitive
        if taken.iter().any(|existing| existing.eq_ignore_ascii_case(name)) {
            return Some(NameProblemView::Taken);
        }
        None
    }
}
