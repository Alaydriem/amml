use amml_i18n::text::Text;

use crate::tokens::selector::StatusColor;

/// One coverage item: its id (as in "A14"), its name, its status, the area it lives in (as in "Library"), and
/// whether "Go" can take the user to it.
#[derive(Debug, Clone)]
pub struct CoverageRow {
    pub id: Text,
    pub label: Text,
    pub status: StatusColor,
    pub area: Text,
    pub go: bool,
}
