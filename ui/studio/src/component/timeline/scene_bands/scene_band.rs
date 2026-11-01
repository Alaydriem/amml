use amml_i18n::text::Text;

use crate::tokens::selector::SceneColor;

/// One scene on the scene band row: its name, its time range as the caller writes it (as in
/// "00:28.80 – 00:37.40"), its start and end in seconds, its colour, and the band row it sits on (0 or 1).
#[derive(Debug, Clone)]
pub struct SceneBand {
    pub name: Text,
    pub range: Text,
    pub start: f64,
    pub end: f64,
    pub color: SceneColor,
    pub row: usize,
    pub active: bool,
}
