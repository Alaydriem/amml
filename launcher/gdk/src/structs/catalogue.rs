use super::{CatalogueEntry, Channel, GameVersion};

#[derive(Debug, Clone)]
pub struct Catalogue {
    release: Vec<CatalogueEntry>,
    preview: Vec<CatalogueEntry>,
}

impl Catalogue {
    // both lists are stored newest first
    pub fn new(mut release: Vec<CatalogueEntry>, mut preview: Vec<CatalogueEntry>) -> Catalogue {
        release.sort_by_key(|entry| std::cmp::Reverse(entry.version));
        preview.sort_by_key(|entry| std::cmp::Reverse(entry.version));
        Catalogue { release, preview }
    }

    pub fn entries(&self, channel: Channel) -> Vec<&CatalogueEntry> {
        match channel {
            Channel::Release => self.release.iter().collect(),
            Channel::Preview => self.preview.iter().collect(),
        }
    }

    pub fn find(&self, version: &GameVersion, channel: Channel) -> Option<&CatalogueEntry> {
        self.entries(channel).into_iter().find(|entry| &entry.version == version)
    }

    pub fn find_by_full_name(&self, full_name: &str) -> Option<&CatalogueEntry> {
        self.release
            .iter()
            .chain(self.preview.iter())
            .find(|entry| entry.package_full_name().is_some_and(|name| name.eq_ignore_ascii_case(full_name)))
    }
}
