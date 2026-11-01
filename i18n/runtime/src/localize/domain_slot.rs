use std::sync::OnceLock;

use crate::pack::{LanguagePack, PackEntry};

use super::PluralCategoryMap;

/// The loaded pack for one domain, and its message table once a caller has asked for it.
///
/// The table is built on first use because only the domain's `Message` enum knows the keys.
#[derive(Debug, Default)]
pub struct DomainSlot {
    pack: Option<LanguagePack>,
    forms: Option<PluralCategoryMap>,
    entries: OnceLock<Vec<Option<PackEntry>>>,
}

impl DomainSlot {
    pub fn new(pack: Option<LanguagePack>) -> Self {
        let forms = pack.as_ref().map(|pack| PluralCategoryMap::new(&pack.plural));
        Self { pack, forms, entries: OnceLock::new() }
    }

    pub fn has_pack(&self) -> bool {
        self.pack.is_some()
    }

    /// The entries indexed by message index; `build` runs once, on the first call.
    pub fn entries(&self, build: fn(&LanguagePack) -> Vec<Option<PackEntry>>) -> &[Option<PackEntry>] {
        match &self.pack {
            Some(pack) => self.entries.get_or_init(|| build(pack)),
            None => &[],
        }
    }

    pub fn forms(&self) -> Option<&PluralCategoryMap> {
        self.forms.as_ref()
    }
}
