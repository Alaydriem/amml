use amml_i18n::text::Text;

/// One choice in a `Dropdown`.
#[derive(Debug, Clone)]
pub struct DropdownOption {
    pub(crate) text: Text,
    pub(crate) enabled: bool,
}

impl DropdownOption {
    pub fn new(text: Text) -> Self {
        Self { text, enabled: true }
    }

    /// A disabled option shows muted and cannot be picked.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
