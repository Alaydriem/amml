use std::collections::HashMap;

use amml_i18n::pack::{LanguagePack, PackEntry};
use amml_i18n::service::LocaleSetting;

use crate::error::ToolError;
use crate::po::{PoCatalog, PoReader};

/// Builds `en_XA` from a .pot file.
///
/// A string that was never marked shows as plain English and cannot be found by reading the
/// screen. Bracketing every marked string leaves the unmarked ones as the only plain text, and
/// padding to roughly German length makes a layout that cannot survive translation fail first.
pub struct PseudoLocale;

impl PseudoLocale {
    const EXPANSION: f64 = 0.4;

    pub fn from_pot(pot: &str) -> Result<LanguagePack, ToolError> {
        Ok(Self::from_catalog(&PoReader::parse(pot)?))
    }

    pub fn from_catalog(catalog: &PoCatalog) -> LanguagePack {
        let mut m = HashMap::new();
        for message in &catalog.messages {
            let entry = match &message.msgid_plural {
                Some(plural) => PackEntry::Many(vec![Self::transform(&message.msgid), Self::transform(plural)]),
                None => PackEntry::One(Self::transform(&message.msgid)),
            };
            m.insert(message.key(), entry);
        }
        LanguagePack {
            v: 1,
            locale: LocaleSetting::PSEUDO_LOCALE.to_owned(),
            plural: vec!["one".to_owned(), "other".to_owned()],
            m,
        }
    }

    // Placeholders pass through untouched: accenting `{count}` would break substitution.
    fn transform(source: &str) -> String {
        let mut accented = String::with_capacity(source.len() * 2);
        let mut rest = source;
        while let Some(open) = rest.find('{') {
            accented.extend(rest[..open].chars().map(Self::accent));
            let after = &rest[open..];
            let close = after.find('}').map_or(after.len(), |close| close + 1);
            accented.push_str(&after[..close]);
            rest = &after[close..];
        }
        accented.extend(rest.chars().map(Self::accent));
        let padding = "~".repeat((source.chars().count() as f64 * Self::EXPANSION).ceil() as usize);
        format!("[{accented} {padding}]")
    }

    fn accent(letter: char) -> char {
        match letter {
            'a' => 'å',
            'b' => 'ƀ',
            'c' => 'ç',
            'd' => 'ð',
            'e' => 'é',
            'f' => 'ƒ',
            'g' => 'ĝ',
            'h' => 'ĥ',
            'i' => 'î',
            'j' => 'ĵ',
            'k' => 'ķ',
            'l' => 'ļ',
            'm' => 'ɱ',
            'n' => 'ñ',
            'o' => 'ô',
            'p' => 'þ',
            'q' => 'ǫ',
            'r' => 'ŕ',
            's' => 'š',
            't' => 'ţ',
            'u' => 'û',
            'v' => 'ṽ',
            'w' => 'ŵ',
            'x' => 'ẋ',
            'y' => 'ý',
            'z' => 'ž',
            'A' => 'Å',
            'B' => 'Ɓ',
            'C' => 'Ç',
            'D' => 'Ð',
            'E' => 'É',
            'F' => 'Ƒ',
            'G' => 'Ĝ',
            'H' => 'Ĥ',
            'I' => 'Ĩ',
            'J' => 'Ĵ',
            'K' => 'Ķ',
            'L' => 'Ļ',
            'M' => 'Ṁ',
            'N' => 'Ñ',
            'O' => 'Ô',
            'P' => 'Þ',
            'Q' => 'Ǫ',
            'R' => 'Ŕ',
            'S' => 'Ṡ',
            'T' => 'Ţ',
            'U' => 'Û',
            'V' => 'Ṽ',
            'W' => 'Ŵ',
            'X' => 'Ẋ',
            'Y' => 'Ý',
            'Z' => 'Ž',
            other => other,
        }
    }
}
