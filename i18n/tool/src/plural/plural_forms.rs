use std::str::FromStr;

use amml_i18n::localize::PluralCategoryMap;
use icu_locale_core::Locale;
use icu_plurals::PluralRules;

use crate::error::ToolError;

use super::PluralExpression;

/// Turns a gettext `Plural-Forms` header into the CLDR plural categories, ordered by the
/// msgstr index each one selects.
///
/// gettext orders forms by an expression; CLDR names them. The two are matched by running
/// the expression over integers and asking ICU what each integer is called. Categories a
/// locale uses only for fractions never appear, which is correct: a gettext catalog has no
/// form for them.
pub struct PluralForms;

impl PluralForms {
    // Far enough to reach every residue class real Plural-Forms expressions test, which
    // stop at n % 100.
    const PROBE_LIMIT: u64 = 200;

    pub fn check(locale: &str, header: &str) -> Result<Vec<String>, ToolError> {
        let (count, expression) = Self::parse_header(header)?;
        let rules = Locale::from_str(&locale.replace('_', "-"))
            .ok()
            .and_then(|parsed| PluralRules::try_new_cardinal((&parsed).into()).ok())
            .ok_or_else(|| ToolError::PluralForms(format!("{locale}: no CLDR plural rules for this locale")))?;
        let mut categories: Vec<Option<&'static str>> = vec![None; count];

        for n in 0..=Self::PROBE_LIMIT {
            let index = PluralExpression::evaluate(&expression, n)?;
            let slot = usize::try_from(index).ok().and_then(|index| categories.get_mut(index)).ok_or_else(|| {
                ToolError::PluralForms(format!(
                    "{locale}: plural expression selected form {index}, outside 0..{}",
                    count.saturating_sub(1)
                ))
            })?;
            let category = PluralCategoryMap::name(rules.category_for(n));
            match slot {
                None => *slot = Some(category),
                Some(known) if *known != category => {
                    return Err(ToolError::PluralForms(format!(
                        "{locale}: plural form {index} maps to both {known:?} and {category:?}"
                    )));
                }
                Some(_) => {}
            }
        }

        let names: Vec<String> = categories
            .iter()
            .enumerate()
            .map(|(index, category)| {
                category
                    .map(str::to_owned)
                    .ok_or_else(|| ToolError::PluralForms(format!("{locale}: plural form {index} is never selected")))
            })
            .collect::<Result<_, _>>()?;

        // The runtime picks a form by category, so two forms with one category would leave
        // the second unreachable. This also catches a locale with no CLDR data, where ICU's
        // root rules call every count `other`.
        for (index, name) in names.iter().enumerate() {
            if let Some(first) = names[..index].iter().position(|earlier| earlier == name) {
                return Err(ToolError::PluralForms(format!(
                    "{locale}: plural forms {first} and {index} both map to {name:?}; use a header with one form per CLDR category"
                )));
            }
        }
        Ok(names)
    }

    fn parse_header(header: &str) -> Result<(usize, String), ToolError> {
        let mut count = None;
        let mut expression = None;
        for part in header.split(';') {
            if let Some((key, value)) = part.split_once('=') {
                match key.trim() {
                    "nplurals" => count = value.trim().parse::<usize>().ok(),
                    "plural" => expression = Some(value.trim().to_owned()),
                    _ => {}
                }
            }
        }
        match (count, expression) {
            (Some(count), Some(expression)) if count > 0 => Ok((count, expression)),
            _ => Err(ToolError::PluralForms(format!("unparseable Plural-Forms header: {header:?}"))),
        }
    }
}
