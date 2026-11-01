use std::collections::{BTreeSet, HashMap};

use amml_i18n::pack::{LanguagePack, PackEntry};

use crate::error::ToolError;
use crate::plural::PluralForms;
use crate::po::{PoCatalog, PoMessage};
use crate::report::Coverage;

use super::PlaceholderScan;

/// Compiles a translated .po catalog into the pack the runtime reads.
pub struct PackCompiler;

impl PackCompiler {
    /// The placeholder a plural form may leave out: some languages spell out "one".
    const COUNT: &'static str = "count";

    /// Untranslated messages are left out, so the runtime shows English for them.
    pub fn compile(catalog: &PoCatalog, locale: &str) -> Result<LanguagePack, ToolError> {
        let plural = PluralForms::check(locale, &catalog.plural_forms)?;
        let mut m = HashMap::new();
        for message in catalog.messages.iter().filter(|message| catalog.is_translated(message)) {
            Self::check_placeholders(message, locale)?;
            let entry = match message.msgid_plural {
                Some(_) => PackEntry::Many(message.msgstr.clone()),
                None => PackEntry::One(message.msgstr.first().cloned().unwrap_or_default()),
            };
            m.insert(message.key(), entry);
        }
        Ok(LanguagePack { v: 1, locale: locale.to_owned(), plural, m })
    }

    pub fn coverage(catalog: &PoCatalog) -> Coverage {
        let translated = catalog.messages.iter().filter(|message| catalog.is_translated(message)).count();
        Coverage::new(translated, catalog.messages.len())
    }

    // A translation that drops or renames a placeholder would show `{name}` on screen or
    // lose the value, so it is refused at build time with the msgid in the error.
    fn check_placeholders(message: &PoMessage, locale: &str) -> Result<(), ToolError> {
        let mut expected = PlaceholderScan::names(&message.msgid);
        if let Some(plural) = &message.msgid_plural {
            expected.extend(PlaceholderScan::names(plural));
        }
        for form in &message.msgstr {
            let found = PlaceholderScan::names(form);
            let matches = match message.msgid_plural {
                Some(_) => {
                    found.is_subset(&expected)
                        && expected.iter().all(|name| *name == Self::COUNT || found.contains(name))
                }
                None => found == expected,
            };
            if !matches {
                return Err(ToolError::Placeholder {
                    locale: locale.to_owned(),
                    msgid: message.msgid.clone(),
                    expected: Self::owned(&expected),
                    found: Self::owned(&found),
                });
            }
        }
        Ok(())
    }

    fn owned(names: &BTreeSet<&str>) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }
}
