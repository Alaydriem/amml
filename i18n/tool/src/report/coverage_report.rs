use std::collections::HashSet;

use amml_i18n::pack::LanguagePack;

use crate::error::ToolError;
use crate::po::{PoCatalog, PoMessage, PoReader};

use super::Coverage;

/// Measures packs against the current .pot and prints the locale table.
pub struct CoverageReport;

impl CoverageReport {
    /// Counts the .pot messages that `pack` translates. Pack entries for messages that left
    /// the .pot do not count.
    pub fn for_pack(pot: &str, pack: &LanguagePack) -> Result<Coverage, ToolError> {
        let catalog = PoReader::parse(pot)?;
        let translated = catalog.messages.iter().filter(|message| pack.m.contains_key(&message.key())).count();
        Ok(Coverage::new(translated, catalog.messages.len()))
    }

    /// Counts the .pot messages that `po` translates in full, without compiling it.
    pub fn for_catalog(pot: &str, po: &PoCatalog) -> Result<Coverage, ToolError> {
        let template = PoReader::parse(pot)?;
        let done: HashSet<String> =
            po.messages.iter().filter(|message| po.is_translated(message)).map(PoMessage::key).collect();
        let translated = template.messages.iter().filter(|message| done.contains(&message.key())).count();
        Ok(Coverage::new(translated, template.messages.len()))
    }

    /// A Markdown table, one row per locale.
    pub fn render(locales: &[(String, Coverage)]) -> String {
        if locales.is_empty() {
            return "### Locales\n\nNo `.po` files yet. Every string falls back to its English source.\n".to_owned();
        }
        let mut lines = vec![
            "### Locales".to_owned(),
            String::new(),
            "| Locale | Translated | Coverage | Ships |".to_owned(),
            "|---|---:|---:|:-:|".to_owned(),
        ];
        for (locale, coverage) in locales {
            lines.push(format!(
                "| `{locale}` | {} / {} | {}% | {} |",
                coverage.translated,
                coverage.total,
                coverage.percent,
                if coverage.ships() { "yes" } else { "no" }
            ));
        }
        lines.push(String::new());
        lines.push(format!("_A locale ships at {}% or above._", Coverage::SHIP_PERCENT));
        lines.join("\n") + "\n"
    }
}
