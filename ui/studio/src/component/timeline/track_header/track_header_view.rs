use amml_i18n::text::Text;

use crate::tokens::selector::TrackColor;

/// What a track header shows.
#[derive(Debug, Clone)]
pub struct TrackHeaderView {
    pub name: Text,
    pub color: TrackColor,
    /// The number of keys, in the count pill.
    pub keys: usize,
    pub hidden: bool,
    pub locked: bool,
    pub selected: bool,
}
