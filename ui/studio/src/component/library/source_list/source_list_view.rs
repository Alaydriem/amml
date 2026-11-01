use amml_i18n::text::Text;

use super::SourceRow;

/// What the Library's source list shows. Every word comes from the host.
pub struct SourceListView {
    /// The title above the rows, written in upper case in the message itself.
    pub title: Text,
    pub sources: Vec<SourceRow>,
    /// The words of the button that adds a folder.
    pub add: Text,
}
