use amml_i18n::text::Text;

use crate::tokens::selector::SceneColor;

/// What a Library scene card shows.
pub struct SceneCardView {
    pub name: Text,
    /// The scene's range on the timeline.
    pub range: Text,
    pub color: SceneColor,
}
