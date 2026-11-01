/// Paper: Widgets, `Widget / Text field`, columns FIELD and SEARCH.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextFieldKind {
    #[default]
    Plain,
    /// A leading search icon and a trailing clear button while there is text.
    Search,
}
