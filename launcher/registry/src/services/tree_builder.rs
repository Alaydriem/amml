use std::collections::BTreeMap;

use crate::errors::RegistryError;
use crate::services::{HashText, JsonText, RegistryUpdater};
use crate::structs::{
    AmmlSection, ApiSummary, FileRef, IndexCard, IndexPage, IndexSummary, ModDocument, NameRow, NamesFile,
    PublishedTree, RegistryRoot, ReleaseVersion,
};

/// Regenerates `registry.json`, `names.json` and every index page from the mod documents.
/// The same documents always give the same bytes.
pub struct TreeBuilder;

impl TreeBuilder {
    pub const PAGE_SIZE: usize = 100;
    pub const NAMES_PATH: &'static str = "names.json";
    pub const ROOT_PATH: &'static str = "registry.json";

    pub fn build(
        name: &str,
        published: &str,
        amml: &AmmlSection,
        documents: &[ModDocument],
    ) -> Result<PublishedTree, RegistryError> {
        let mut sorted: Vec<&ModDocument> = documents.iter().collect();
        sorted.sort_by(|a, b| a.id.to_lowercase().cmp(&b.id.to_lowercase()).then_with(|| a.id.cmp(&b.id)));
        for pair in sorted.windows(2) {
            if pair[0].id == pair[1].id {
                return Err(RegistryError::BadTree(format!("{} appears twice", pair[0].id)));
            }
        }

        // cards per API version, in id order
        let mut by_api: BTreeMap<u32, Vec<IndexCard>> = BTreeMap::new();
        // for each mod id: the APIs it has, newest first
        let mut apis_of: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
        for document in &sorted {
            let mut newest: BTreeMap<u32, (ReleaseVersion, usize)> = BTreeMap::new();
            for (position, release) in document.versions.iter().enumerate() {
                let version = ReleaseVersion::parse(&release.version)?;
                let slot = newest.entry(release.api_version).or_insert((version, position));
                if version > slot.0 {
                    *slot = (version, position);
                }
            }
            let mut apis: Vec<u32> = newest.keys().copied().collect();
            apis.sort_by(|a, b| b.cmp(a));
            for (api, (_, position)) in &newest {
                let release = &document.versions[*position];
                by_api.entry(*api).or_default().push(IndexCard {
                    id: document.id.clone(),
                    title: document.listing.title.clone(),
                    summary: document.listing.summary.clone(),
                    version: release.version.clone(),
                    size: release.size,
                    icon: document.listing.icon.clone(),
                    core: document.core,
                    default: document.default,
                    updated: release.published.clone(),
                });
            }
            apis_of.insert(document.id.as_str(), apis);
        }

        let mut tree = PublishedTree::default();
        let mut summary = IndexSummary { page_size: Self::PAGE_SIZE as u32, apis: BTreeMap::new() };
        let mut page_of: BTreeMap<(u32, &str), u32> = BTreeMap::new();
        for (api, cards) in &by_api {
            let pages = cards.chunks(Self::PAGE_SIZE);
            summary.apis.insert(api.to_string(), ApiSummary { mods: cards.len() as u32, pages: pages.len() as u32 });
            for (position, chunk) in pages.enumerate() {
                let number = position as u32 + 1;
                for card in chunk {
                    page_of.insert((*api, card.id.as_str()), number);
                }
                let page = IndexPage {
                    schema: RegistryUpdater::SCHEMA,
                    api_version: *api,
                    page: number,
                    cards: chunk.to_vec(),
                };
                tree.files.insert(format!("index/api-{api}/{number:04}.json"), JsonText::compact(&page)?);
            }
        }

        let rows = sorted
            .iter()
            .map(|document| {
                let apis = apis_of.get(document.id.as_str()).cloned().unwrap_or_default();
                let page =
                    apis.first().and_then(|api| page_of.get(&(*api, document.id.as_str()))).copied().unwrap_or(1);
                NameRow(document.id.clone(), document.listing.title.clone(), document.listing.tags.clone(), apis, page)
            })
            .collect();
        let names = JsonText::compact(&NamesFile { schema: RegistryUpdater::SCHEMA, rows })?;

        let root = RegistryRoot {
            schema: RegistryUpdater::SCHEMA,
            name: name.to_string(),
            published: published.to_string(),
            mod_count: sorted.len() as u32,
            amml: amml.clone(),
            index: summary,
            names: FileRef {
                path: Self::NAMES_PATH.to_string(),
                sha256: HashText::sha256(&names),
                size: names.len() as u64,
            },
        };
        tree.files.insert(Self::ROOT_PATH.to_string(), JsonText::pretty(&root)?);
        tree.files.insert(Self::NAMES_PATH.to_string(), names);
        Ok(tree)
    }
}
