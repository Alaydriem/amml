use amml_i18n::text::Text;

use crate::tokens::selector::ModeColor;

/// What the status bar shows.
pub struct StatusBarView {
    /// The mode dot's colour.
    pub mode: ModeColor,
    /// The mode, as "Edit · free camera".
    pub mode_text: Text,
    /// The rewind cache and seek detail; left out in Compact.
    pub detail: Text,
    /// The fly speed, UI scale and key hints, before the legend.
    pub shortcuts: Text,
    /// The bar has the preview tint (the scene camera drives).
    pub tinted: bool,
}
