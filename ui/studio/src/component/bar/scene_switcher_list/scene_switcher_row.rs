use amml_i18n::text::Text;

use crate::tokens::selector::SceneColor;

/// One scene in the scene switcher list.
pub struct SceneSwitcherRow {
    pub name: Text,
    /// The scene's range on the timeline.
    pub range: Text,
    /// How many camera keys the scene has.
    pub cameras: Text,
    /// The status pill's words (Draft, Ready, Rendered).
    pub status: Text,
    pub color: SceneColor,
    /// The scene being edited: a soft accent row with a check.
    pub active: bool,
}
