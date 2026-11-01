use super::PoMessage;

/// A parsed .po or .pot file: its `Plural-Forms` header and its messages, header excluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoCatalog {
    pub plural_forms: String,
    /// The number of forms the header declares.
    pub nplurals: usize,
    pub messages: Vec<PoMessage>,
}

impl PoCatalog {
    // A plural entry with fewer forms than the header declares would make the runtime show
    // its last form for every missing category, so it does not count as translated.
    pub fn is_translated(&self, message: &PoMessage) -> bool {
        message.is_translated() && (message.msgid_plural.is_none() || message.msgstr.len() == self.nplurals)
    }
}
