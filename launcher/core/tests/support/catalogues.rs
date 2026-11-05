use super::{FakeServer, RegistryTree, Route, Zips};
use amml_gdk::Downloader;
use amml_launcher_core::{
    AssetFetcher, CacheLock, ModCatalogue, RegistryCache, RegistryClient, RegistryRef, RegistrySet, UreqHttpFetch,
};
use amml_registry::{HashText, ModRelease};
use std::path::Path;

// Catalogues over fake registry servers, and mod zips served from a fake server.
pub struct Catalogues;

impl Catalogues {
    pub fn client(server: &FakeServer, official: bool, cache: &Path) -> RegistryClient<UreqHttpFetch> {
        let registry = if official { RegistryRef::official_at(server.base()) } else { RegistryRef::new(server.base()) };
        Self::client_for(registry, cache)
    }

    pub fn client_for(registry: RegistryRef, cache: &Path) -> RegistryClient<UreqHttpFetch> {
        RegistryClient::new(
            registry,
            UreqHttpFetch::new(UreqHttpFetch::agent()),
            RegistryCache::new(cache.to_path_buf(), RegistryCache::BUDGET, CacheLock::new()),
        )
    }

    pub fn catalogue(clients: Vec<RegistryClient<UreqHttpFetch>>, downloads: &Path) -> ModCatalogue<UreqHttpFetch> {
        let downloader =
            Downloader::new(UreqHttpFetch::agent(), downloads.to_path_buf(), 1, amml_gdk::CancelFlag::new());
        ModCatalogue::new(RegistrySet::new(clients), AssetFetcher::new(downloader, downloads.to_path_buf()))
    }

    // builds a mod zip, serves it at `/zips/<ns>-<name>-<version>.zip` and returns the release that describes it
    pub fn serve_mod(
        server: &FakeServer,
        work: &Path,
        namespace: &str,
        name: &str,
        version: &str,
        api: u32,
        extra: &[(&str, &[u8])],
    ) -> ModRelease {
        let zip = Zips::mod_zip(
            &work.join(format!("{namespace}-{name}-{version}.zip")),
            namespace,
            name,
            version,
            api,
            extra,
        );
        let bytes = std::fs::read(&zip).unwrap();
        let path = format!("/zips/{namespace}-{name}-{version}.zip");
        server.replace(&path, Route::new(bytes.clone()));
        RegistryTree::release(version, api, &server.url(&path), &HashText::sha256(&bytes), bytes.len() as u64)
    }
}
