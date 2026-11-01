use amml_i18n::pack::ContextKey;

/// One message of a catalog. A singular message has one `msgstr`; a plural one has one per form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoMessage {
    pub context: Option<String>,
    pub msgid: String,
    pub msgid_plural: Option<String>,
    pub msgstr: Vec<String>,
    pub fuzzy: bool,
}

impl PoMessage {
    // A plural message counts only when every form is filled; a partly translated plural
    // would show a blank string for whichever count hits the empty form. A fuzzy entry is a
    // guess the translator has not confirmed, and gettext does not ship it either.
    pub fn is_translated(&self) -> bool {
        !self.fuzzy && !self.msgstr.is_empty() && self.msgstr.iter().all(|form| !form.is_empty())
    }

    /// The key the pack stores this message under.
    pub fn key(&self) -> String {
        ContextKey::join(self.context.as_deref(), &self.msgid)
    }
}
