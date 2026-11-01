use amml_i18n::text::Text;

use crate::tokens::selector::SceneColor;

/// Paper: `3TE-0` section B. What the camera shows: a scene (its name, colour and place, as in "2 of 5"), or the
/// Original replay as recorded.
#[derive(Debug, Clone)]
pub enum ViewerSource {
    Scene { name: Text, color: SceneColor, index: usize, count: usize },
    Original,
}
