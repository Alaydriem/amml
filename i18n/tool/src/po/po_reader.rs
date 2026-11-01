use std::path::Path;

use polib::catalog::Catalog;

use crate::error::ToolError;

use super::{PoCatalog, PoMessage};

/// Reads .po and .pot files with `polib`.
pub struct PoReader;

impl PoReader {
    pub fn read(path: &Path) -> Result<PoCatalog, ToolError> {
        let text =
            std::fs::read_to_string(path).map_err(|source| ToolError::Io { path: path.to_path_buf(), source })?;
        Self::parse_from(&text, &path.display().to_string())
    }

    pub fn parse(text: &str) -> Result<PoCatalog, ToolError> {
        Self::parse_from(text, "<text>")
    }

    // polib 0.3 panics on some typos, such as a stray quote. A .po file comes from outside
    // people, so the panic becomes an error that names the file.
    fn parse_from(text: &str, origin: &str) -> Result<PoCatalog, ToolError> {
        let parsed = std::panic::catch_unwind(|| polib::po_file::parse_from_reader(text.as_bytes()))
            .map_err(|_| ToolError::Catalog { origin: origin.to_owned(), message: "malformed quoting".to_owned() })?;
        let catalog =
            parsed.map_err(|error| ToolError::Catalog { origin: origin.to_owned(), message: error.to_string() })?;
        Ok(Self::convert(&catalog))
    }

    fn convert(catalog: &Catalog) -> PoCatalog {
        let messages = catalog
            .messages()
            .filter(|message| !message.msgid().is_empty())
            .map(|message| PoMessage {
                context: message.msgctxt().map(str::to_owned),
                msgid: message.msgid().to_owned(),
                msgid_plural: message.msgid_plural().ok().map(str::to_owned),
                msgstr: match message.msgstr_plural() {
                    Ok(forms) => forms.clone(),
                    Err(_) => message.msgstr().map(|text| vec![text.to_owned()]).unwrap_or_default(),
                },
                fuzzy: message.is_fuzzy(),
            })
            .collect();
        let rules = &catalog.metadata.plural_rules;
        PoCatalog { plural_forms: rules.dump(), nplurals: rules.nplurals, messages }
    }
}
