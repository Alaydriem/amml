use amml_i18n::text::Text;

/// The last line of a replay row: where the replay comes from, or why it cannot be opened.
pub enum ReplayRowNote {
    /// The source and its flags ("BVC", "recent"), drawn with a dot between them.
    Source(Vec<Text>),
    /// Why the replay is broken. The row draws it with a warning icon in the danger colour.
    Broken(Text),
}
