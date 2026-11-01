use amml_i18n::text::Text;

/// What kind of source a row of the source list is.
pub enum SourceRowKind {
    /// Found on its own; it has a small tag after the name and no remove button.
    Automatic { tag: Text },
    /// A folder the person added; it has a remove button.
    Folder,
}
