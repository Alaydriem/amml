use std::ops::Range;

use amml_i18n::text::Text;

/// How a text-entry field looks beyond its interaction state, for painting it by hand (the gallery
/// draws the TYPING and INVALID cells with this). A live field fills it from its own edit state.
#[derive(Debug, Clone, Default)]
pub struct FieldLook {
    /// Selected characters of the value; the caret sits at the end. `None` shows no caret.
    pub selection: Option<Range<usize>>,
    /// Why the value is invalid, shown under the field with the danger border.
    pub invalid: Option<Text>,
}
