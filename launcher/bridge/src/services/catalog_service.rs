use super::BrowseLayout;
use crate::{BridgeError, CardStateView, CardView, CatalogLayoutView, CatalogSortView, ModDetailView, SearchGroupView};
use amml_launcher_core::{Compatibility, HttpFetch, LauncherError, ModId, NameSearch, RegistryClient, RegistrySet};
use amml_registry::{IndexCard, NameRow};
use std::collections::{BTreeMap, BTreeSet};

// Reads a registry's index for the browse and search screens. It holds nothing between calls: the registry clients
// keep the pages (at most ten each) and the disk cache.
pub struct CatalogService;

impl CatalogService {
    // the most cards a registry gives to a search
    pub const SEARCH_LIMIT: usize = 50;
    // a registry with more pages than this for the API is not sorted by anything but name
    pub const SORT_PAGE_LIMIT: u32 = 10;

    // The shape of a registry's index for one API. Page numbers count from 1: the page that holds the row at index
    // `i` is `i / page_size + 1`.
    pub fn layout<H: HttpFetch>(
        set: &mut RegistrySet<H>,
        registry_id: &str,
        api: u32,
    ) -> Result<CatalogLayoutView, BridgeError> {
        let client = Self::client(set, registry_id)?;
        let root = Self::root(client)?;
        let names = client.names()?.value;
        let groups = BrowseLayout::build(&names.rows, api);
        let total = groups.iter().map(|group| group.total).sum();
        Ok(CatalogLayoutView {
            registry_id: registry_id.to_string(),
            api,
            page_size: root.index.page_size,
            total,
            groups,
        })
    }

    pub fn page<H: HttpFetch>(
        set: &mut RegistrySet<H>,
        registry_id: &str,
        api: u32,
        page: u32,
        installed: &[(String, Option<String>)],
    ) -> Result<Vec<CardView>, BridgeError> {
        let client = Self::client(set, registry_id)?;
        let base = client.registry().base.clone();
        let cached = client.page(api, page)?;
        Ok(cached.value.cards.iter().map(|card| Self::card(card, registry_id, &base, installed)).collect())
    }

    // The best hits of every registry that answers. A registry that cannot be read drops out and the others answer.
    pub fn search<H: HttpFetch>(
        set: &mut RegistrySet<H>,
        query: &str,
        api: u32,
        installed: &[(String, Option<String>)],
    ) -> Vec<SearchGroupView> {
        let mut groups = Vec::new();
        for client in set.clients_mut() {
            match Self::search_one(client, query, api, installed) {
                Ok(Some(group)) => groups.push(group),
                Ok(None) => {}
                Err(error) => {
                    curia::warn!("a registry dropped out of a search", {
                        registry: client.registry().base.clone(),
                        error: error.to_string(),
                    });
                }
            }
        }
        groups
    }

    // Every card of the registry for the API, sorted. `None` for the name order (the index is already in it) and
    // for a registry with more than `SORT_PAGE_LIMIT` pages.
    pub fn sorted<H: HttpFetch>(
        set: &mut RegistrySet<H>,
        registry_id: &str,
        api: u32,
        sort: CatalogSortView,
        installed: &[(String, Option<String>)],
    ) -> Result<Option<Vec<CardView>>, BridgeError> {
        if sort == CatalogSortView::Name {
            return Ok(None);
        }
        let client = Self::client(set, registry_id)?;
        let root = Self::root(client)?;
        let base = client.registry().base.clone();
        let pages = root.index.apis.get(&api.to_string()).map_or(0, |summary| summary.pages);
        if pages > Self::SORT_PAGE_LIMIT {
            return Ok(None);
        }
        let mut cards: Vec<IndexCard> = Vec::new();
        for number in 1..=pages {
            cards.extend(client.page(api, number)?.value.cards);
        }
        match sort {
            CatalogSortView::Updated => cards.sort_by(|left, right| right.updated.cmp(&left.updated)),
            _ => cards.sort_by_key(|card| std::cmp::Reverse(card.size)),
        }
        Ok(Some(cards.iter().map(|card| Self::card(card, registry_id, &base, installed)).collect()))
    }

    pub fn detail<H: HttpFetch>(
        set: &mut RegistrySet<H>,
        registry_id: &str,
        id: &str,
        api: u32,
    ) -> Result<ModDetailView, BridgeError> {
        let client = Self::client(set, registry_id)?;
        let base = client.registry().base.clone();
        let cached = client.document(&ModId::parse(id)?)?;
        let document = cached.value;
        let release = Compatibility::newest_for_api(&document, api)
            .ok_or_else(|| LauncherError::UnknownMod(document.id.clone()))?;
        let listing = &document.listing;
        let card = CardView {
            id: document.id.clone(),
            registry_id: registry_id.to_string(),
            title: listing.title.clone(),
            summary: listing.summary.clone(),
            tags: listing.tags.clone(),
            version: release.version.clone(),
            size: release.size,
            icon_url: listing.icon.as_ref().map(|icon| format!("{base}{icon}")),
            state: CardStateView::Available,
            updated: release.published.clone(),
        };
        Ok(ModDetailView {
            card,
            description: listing.description.clone(),
            authors: listing.authors.clone(),
            license: listing.license.clone(),
            source: listing.source.clone(),
            homepage: listing.homepage.clone(),
            api,
            sha256: release.sha256,
            stale: cached.stale,
        })
    }

    fn client<'a, H: HttpFetch>(
        set: &'a mut RegistrySet<H>,
        registry_id: &str,
    ) -> Result<&'a mut RegistryClient<H>, BridgeError> {
        set.clients_mut()
            .iter_mut()
            .find(|client| client.registry().cache_key() == registry_id)
            .ok_or_else(|| BridgeError::Unavailable(registry_id.to_string()))
    }

    // the root of the registry, asking for it when the client has none yet
    fn root<H: HttpFetch>(client: &mut RegistryClient<H>) -> Result<amml_registry::RegistryRoot, BridgeError> {
        if client.root().is_none() {
            client.refresh();
        }
        client
            .root()
            .cloned()
            .ok_or_else(|| BridgeError::Launcher(LauncherError::Offline { url: client.registry().base.clone() }))
    }

    fn search_one<H: HttpFetch>(
        client: &mut RegistryClient<H>,
        query: &str,
        api: u32,
        installed: &[(String, Option<String>)],
    ) -> Result<Option<SearchGroupView>, BridgeError> {
        let root = Self::root(client)?;
        let names = client.names()?.value;
        let hits = NameSearch::search(&names, query);
        // the index of each row that lists the API, in id order, so a row's page is known without asking for it
        let ordinals: BTreeMap<&str, usize> = names
            .rows
            .iter()
            .filter(|row| row.3.contains(&api))
            .enumerate()
            .map(|(ordinal, row): (usize, &NameRow)| (row.0.as_str(), ordinal))
            .collect();
        let hits: Vec<_> = hits.into_iter().filter(|hit| ordinals.contains_key(hit.id.as_str())).collect();
        if hits.is_empty() {
            return Ok(None);
        }
        let total = hits.len() as u32;
        let kept = &hits[..hits.len().min(Self::SEARCH_LIMIT)];
        let page_size = root.index.page_size.max(1) as usize;
        let pages: BTreeSet<u32> = kept
            .iter()
            .filter_map(|hit| ordinals.get(hit.id.as_str()))
            .map(|ordinal| (ordinal / page_size) as u32 + 1)
            .collect();
        let base = client.registry().base.clone();
        let registry_id = client.registry().cache_key();
        let mut found: BTreeMap<String, IndexCard> = BTreeMap::new();
        for page in pages {
            for card in client.page(api, page)?.value.cards {
                found.insert(card.id.clone(), card);
            }
        }
        let tags: BTreeMap<&str, &Vec<String>> = names.rows.iter().map(|row| (row.0.as_str(), &row.2)).collect();
        let mut cards = Vec::new();
        for hit in kept {
            if let Some(card) = found.get(&hit.id) {
                let mut view = Self::card(card, &registry_id, &base, installed);
                view.tags = tags.get(hit.id.as_str()).map(|held| (*held).clone()).unwrap_or_default();
                cards.push(view);
            }
        }
        Ok(Some(SearchGroupView { registry_id, total, cards }))
    }

    fn card(card: &IndexCard, registry_id: &str, base: &str, installed: &[(String, Option<String>)]) -> CardView {
        let state = match installed.iter().find(|(id, _)| id == &card.id) {
            None => CardStateView::Available,
            Some((_, Some(version))) if version != &card.version => CardStateView::Update,
            Some(_) => CardStateView::Installed,
        };
        CardView {
            id: card.id.clone(),
            registry_id: registry_id.to_string(),
            title: card.title.clone(),
            summary: card.summary.clone(),
            tags: Vec::new(),
            version: card.version.clone(),
            size: card.size,
            icon_url: card.icon.as_ref().map(|icon| format!("{base}{icon}")),
            state,
            updated: card.updated.clone(),
        }
    }
}
