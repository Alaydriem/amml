use amml_i18n::text::Text;

use super::SourceRowKind;

/// One source of the source list: where replays are found.
pub struct SourceRow {
    pub name: Text,
    pub kind: SourceRowKind,
}
