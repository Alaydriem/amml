use std::str::FromStr;

use icu_locale_core::Locale;
use icu_plurals::{PluralCategory, PluralRules};

use crate::message::{Domain, Message};
use crate::pack::{LanguagePack, PackEntry};

use super::{DomainSlot, MessageTable};

/// The active locale: its plural rules and one pack slot per loaded domain.
pub struct Localizer {
    locale: Option<String>,
    rules: Option<PluralRules>,
    // A mod loads a few domains, so a linear search beats hashing the name.
    slots: Vec<(Domain, DomainSlot)>,
}

impl Localizer {
    /// English needs no pack: every message shows its msgid.
    pub fn english() -> Self {
        Self { locale: None, rules: None, slots: Vec::new() }
    }

    /// A localizer for `locale`. When `packs` names a domain twice, the last pack wins.
    pub fn new(locale: &str, packs: Vec<(Domain, LanguagePack)>) -> Self {
        // A valid tag with no CLDR data gets ICU's root rules, which call 1 `other`. Every
        // locale whose 1 is truly `other` (ja, zh, ko, ...) has a single form, which the
        // one/other fallback also selects, so such rules are dropped.
        let rules = Locale::from_str(&locale.replace('_', "-"))
            .ok()
            .and_then(|parsed| PluralRules::try_new_cardinal((&parsed).into()).ok())
            .filter(|rules| rules.category_for(1u64) != PluralCategory::Other);
        let mut slots: Vec<(Domain, DomainSlot)> = Vec::with_capacity(packs.len());
        for (domain, pack) in packs {
            slots.retain(|(known, _)| *known != domain);
            slots.push((domain, DomainSlot::new(Some(pack))));
        }
        Self { locale: Some(locale.to_owned()), rules, slots }
    }

    /// The locale this localizer was built for; `None` for English.
    pub fn locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }

    pub fn has_pack(&self, domain: Domain) -> bool {
        self.slot(domain).is_some_and(DomainSlot::has_pack)
    }

    fn slot(&self, domain: Domain) -> Option<&DomainSlot> {
        self.slots.iter().find(|(known, _)| *known == domain).map(|(_, slot)| slot)
    }

    /// The CLDR category for a count. With no rules for the locale, 1 is `one` and every
    /// other count is `other`.
    pub fn category(&self, n: u64) -> PluralCategory {
        match &self.rules {
            Some(rules) => rules.category_for(n),
            None if n == 1 => PluralCategory::One,
            None => PluralCategory::Other,
        }
    }

    pub fn table<M: Message>(&self) -> MessageTable<'_, M> {
        MessageTable::new(self)
    }

    /// The translated text for a message, or `None` when the pack has no entry for it.
    pub(crate) fn translated(
        &self,
        domain: Domain,
        index: usize,
        build: fn(&LanguagePack) -> Vec<Option<PackEntry>>,
        count: Option<u64>,
    ) -> Option<&str> {
        let slot = self.slot(domain)?;
        match slot.entries(build).get(index)?.as_ref()? {
            PackEntry::One(text) => Some(text.as_str()),
            PackEntry::Many(forms) => {
                let form = match (count, slot.forms()) {
                    (Some(n), Some(map)) => map.index(self.category(n)),
                    _ => 0,
                };
                forms.get(form.min(forms.len().saturating_sub(1))).map(String::as_str)
            }
        }
    }
}
