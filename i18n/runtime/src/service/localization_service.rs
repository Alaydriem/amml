use crate::localize::Localizer;
use crate::message::Domain;
use crate::negotiate::LocaleNegotiator;
use crate::pack::LanguagePack;

use super::{LocaleSetting, PackSource};

/// Loads the packs for a locale setting from each domain's source.
///
/// English is absent by design: message ids are the English text, so "no pack" is a complete
/// answer rather than a failure.
pub struct LocalizationService {
    sources: Vec<(Domain, PackSource)>,
}

impl LocalizationService {
    pub fn new(sources: Vec<(Domain, PackSource)>) -> Self {
        Self { sources }
    }

    /// The locales shipped for `domain`, sorted.
    pub fn available(&self, domain: Domain) -> Vec<String> {
        self.source(domain).map(PackSource::locales).unwrap_or_default()
    }

    /// A localizer for `setting`. It never fails: a missing or broken pack leaves that domain
    /// in English, and a broken pack logs one warning.
    pub fn load(&self, setting: &LocaleSetting) -> Localizer {
        let Some(requested) = setting.requested() else {
            return Localizer::english();
        };

        let mut locale = None;
        let mut packs = Vec::new();
        for (domain, source) in &self.sources {
            let Some(resolved) = LocaleNegotiator::resolve(requested, &source.locales()) else {
                continue;
            };
            if let Some(pack) = Self::parse(*domain, source, &resolved) {
                locale.get_or_insert(resolved);
                packs.push((*domain, pack));
            }
        }

        match locale {
            Some(locale) => Localizer::new(&locale, packs),
            None => Localizer::english(),
        }
    }

    fn source(&self, domain: Domain) -> Option<&PackSource> {
        self.sources.iter().find(|(known, _)| *known == domain).map(|(_, source)| source)
    }

    fn parse(domain: Domain, source: &PackSource, locale: &str) -> Option<LanguagePack> {
        let parsed = source
            .read(locale)
            .and_then(|bytes| serde_json::from_slice::<LanguagePack>(&bytes).map_err(|error| error.to_string()));
        match parsed {
            Ok(pack) => Some(pack),
            Err(error) => {
                curia::warn!("language pack is unreadable; using English", {
                    domain: domain.name(),
                    locale: locale,
                    error: error,
                });
                None
            }
        }
    }
}
