use amml_i18n::text::Text;

use crate::tokens::selector::RealmColor;

/// A stretch of the replay spent in one realm: its start and end in seconds, its realm and the realm's name.
#[derive(Debug, Clone)]
pub struct RealmSegment {
    pub start: f64,
    pub end: f64,
    pub realm: RealmColor,
    pub name: Text,
}
