use crate::{
    Asset, AssetFetcher, Cached, Compatibility, DefaultMods, HttpFetch, LauncherError, ModId, RegistryClient,
    RegistrySet, ResolvedMod, VerifiedFile,
};
use amml_gdk::DownloadProgress;
use amml_registry::{AmmlRelease, ModDocument};
use std::collections::HashSet;

// Finds mods across the registries and downloads what the player picks.
pub struct ModCatalogue<H: HttpFetch> {
    registries: RegistrySet<H>,
    fetcher: AssetFetcher,
}

impl<H: HttpFetch> ModCatalogue<H> {
    pub fn new(registries: RegistrySet<H>, fetcher: AssetFetcher) -> ModCatalogue<H> {
        ModCatalogue { registries, fetcher }
    }

    pub fn registries_mut(&mut self) -> &mut RegistrySet<H> {
        &mut self.registries
    }

    // The first registry, in settings order, whose document has a release for the API.
    pub fn resolve(&mut self, id: &ModId, api: u32) -> Result<ResolvedMod, LauncherError> {
        for client in self.registries.clients_mut() {
            match client.document(id) {
                Ok(cached) => {
                    if let Some(resolved) = Self::pick(client, cached, api) {
                        return Ok(resolved);
                    }
                }
                Err(error) => {
                    curia::debug!("a registry has no document", {
                        registry: client.registry().base.clone(),
                        mod_id: id.as_str(),
                        error: error.to_string(),
                    });
                }
            }
        }
        Err(LauncherError::UnknownMod(id.as_str()))
    }

    // Like `resolve`, but only in the registry with this base URL (the registry a mod came from).
    pub fn resolve_in(&mut self, base: &str, id: &ModId, api: u32) -> Result<ResolvedMod, LauncherError> {
        for client in self.registries.clients_mut() {
            if client.registry().base != base {
                continue;
            }
            let cached = client.document(id)?;
            return Self::pick(client, cached, api).ok_or_else(|| LauncherError::UnknownMod(id.as_str()));
        }
        Err(LauncherError::UnknownMod(id.as_str()))
    }

    // Every mod marked `default`, and every `core` mod of the official registry, that has a release for the API.
    // The first registry that lists an id decides about it. A page or a core mod's document of the official registry
    // that cannot be read is an error, because a core mod must not be left out unseen. In any other registry a
    // failure only skips what it hides.
    pub fn defaults_for(&mut self, api: u32) -> Result<DefaultMods, LauncherError> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut found = DefaultMods::default();
        for client in self.registries.clients_mut() {
            let official = client.registry().is_official();
            if client.root().is_none() {
                client.refresh();
            }
            let Some(root) = client.root().cloned() else {
                if official {
                    return Err(LauncherError::Offline { url: client.registry().base.clone() });
                }
                continue;
            };
            let Some(summary) = root.index.apis.get(&api.to_string()) else {
                continue;
            };
            for page in 1..=summary.pages {
                let cards = match client.page(api, page) {
                    Ok(cached) => cached.value.cards,
                    Err(error) if official => return Err(error),
                    Err(error) => {
                        curia::warn!("an index page was not read", {
                            registry: client.registry().base.clone(),
                            page: page,
                            error: error.to_string(),
                        });
                        break;
                    }
                };
                for card in cards {
                    if !seen.insert(card.id.clone()) || !(card.default || (official && card.core)) {
                        continue;
                    }
                    let Ok(id) = ModId::parse(&card.id) else {
                        continue;
                    };
                    match client.document(&id) {
                        Ok(cached) => {
                            if let Some(resolved) = Self::pick(client, cached, api) {
                                found.mods.push(resolved);
                            }
                        }
                        Err(error) if official && card.core => return Err(error),
                        Err(error) => found.skipped.push((card.id.clone(), error.to_string())),
                    }
                }
            }
        }
        Ok(found)
    }

    // downloads the release; the registry's SHA-256 and size are checked, stale document or not
    pub fn fetch(
        &self,
        resolved: &ResolvedMod,
        progress: &mut impl FnMut(DownloadProgress),
    ) -> Result<VerifiedFile, LauncherError> {
        let asset = Asset {
            url: resolved.release.asset.clone(),
            sha256: Some(resolved.release.sha256.clone()),
            size: Some(resolved.release.size),
        };
        self.fetcher.fetch(&asset, progress)
    }

    pub fn fetch_amml(
        &self,
        release: &AmmlRelease,
        progress: &mut impl FnMut(DownloadProgress),
    ) -> Result<VerifiedFile, LauncherError> {
        let asset =
            Asset { url: release.asset.clone(), sha256: Some(release.sha256.clone()), size: Some(release.size) };
        self.fetcher.fetch(&asset, progress)
    }

    pub fn fetch_unchecked(
        &self,
        url: &str,
        progress: &mut impl FnMut(DownloadProgress),
    ) -> Result<VerifiedFile, LauncherError> {
        self.fetcher.fetch(&Asset { url: url.to_string(), sha256: None, size: None }, progress)
    }

    pub fn discard(&self, file: &VerifiedFile) {
        self.fetcher.discard(file);
    }

    fn pick(client: &RegistryClient<H>, cached: Cached<ModDocument>, api: u32) -> Option<ResolvedMod> {
        let release = Compatibility::newest_for_api(&cached.value, api)?;
        let registry = client.registry().clone();
        let core = cached.value.core && registry.is_official();
        Some(ResolvedMod { registry, document: cached.value, release, stale: cached.stale, core })
    }
}
