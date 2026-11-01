use amml_i18n::text::Text;

use super::{SceneSwitcherFooter, SceneSwitcherOriginal, SceneSwitcherRow};

/// What the open scene switcher shows. Every word comes from the host.
pub struct SceneSwitcherListView {
    /// The small head above the rows, drawn as given (the host's message is already in the case it should show).
    pub header: Text,
    /// The hint at the right of the head.
    pub hint: Text,
    pub original: SceneSwitcherOriginal,
    pub scenes: Vec<SceneSwitcherRow>,
    pub footer: Vec<SceneSwitcherFooter>,
}
