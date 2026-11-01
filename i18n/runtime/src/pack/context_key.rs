/// Builds the key a pack stores a message under.
///
/// gettext puts U+0004 between context and message id inside compiled catalogs. The runtime
/// and the pack compiler build the same key, so it is part of the pack's contract.
pub struct ContextKey;

impl ContextKey {
    pub const SEPARATOR: char = '\u{4}';

    pub fn join(context: Option<&str>, msgid: &str) -> String {
        match context {
            Some(context) => format!("{context}{}{msgid}", Self::SEPARATOR),
            None => msgid.to_owned(),
        }
    }
}
