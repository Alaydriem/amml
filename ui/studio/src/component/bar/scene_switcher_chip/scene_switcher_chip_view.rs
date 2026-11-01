use amml_i18n::text::Text;

use crate::tokens::selector::SceneColor;

/// What the scene switcher chip shows.
pub struct SceneSwitcherChipView {
    /// The active scene's name and colour, or `None` for the Original replay.
    pub scene: Option<(Text, SceneColor)>,
    /// The active scene's place and the scene count, as (1, 5) for "1 of 5".
    pub position: Option<(usize, usize)>,
    /// The scene switcher menu is open.
    pub open: bool,
}
