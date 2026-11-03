use crate::{
    Cached, Fetched, FileKind, HttpFetch, LauncherError, ModId, RawFile, RegistryCache, RegistryRef, RegistryStatus,
};
use amml_registry::{HashText, IndexPage, ModDocument, NamesFile, RegistryRoot};
use serde::de::DeserializeOwned;
use std::collections::VecDeque;

// One registry: its root, index pages, names, mod documents and icons, over the network with a disk cache behind it.
pub struct RegistryClient<H: HttpFetch> {
    registry: RegistryRef,
    http: H,
    cache: RegistryCache,
    root: Option<RegistryRoot>,
    status: RegistryStatus,
    // least recently used first; at most `PAGES_IN_MEMORY` entries
    pages: VecDeque<((u32, u32), Cached<IndexPage>)>,
    names: Option<Cached<NamesFile>>,
}

impl<H: HttpFetch> RegistryClient<H> {
    const PAGES_IN_MEMORY: usize = 10;
    const SCHEMA: u32 = 2;
    const ROOT_PATH: &'static str = "registry.json";

    pub fn new(registry: RegistryRef, http: H, cache: RegistryCache) -> RegistryClient<H> {
        RegistryClient {
            registry,
            http,
            cache,
            root: None,
            status: RegistryStatus::Offline,
            pages: VecDeque::new(),
            names: None,
        }
    }

    pub fn registry(&self) -> &RegistryRef {
        &self.registry
    }

    pub fn root(&self) -> Option<&RegistryRoot> {
        self.root.as_ref()
    }

    pub fn status(&self) -> &RegistryStatus {
        &self.status
    }

    // the registry's own name, or its base URL while that is unknown
    pub fn name(&self) -> String {
        self.root.as_ref().map(|root| root.name.clone()).unwrap_or_else(|| self.registry.base.clone())
    }

    pub fn pages_in_memory(&self) -> usize {
        self.pages.len()
    }

    pub fn refresh(&mut self) -> RegistryStatus {
        let url = self.registry.url(Self::ROOT_PATH);
        let outcome = match self.load(Self::ROOT_PATH, FileKind::Root) {
            Ok(raw) => self.accept_root(&url, raw),
            Err(LauncherError::Http { text, .. }) => match self.cache.read(&self.registry, Self::ROOT_PATH) {
                Some((bytes, etag)) => {
                    let raw = RawFile { bytes, etag, stale: true, fresh: false };
                    self.accept_root(&url, raw)
                }
                None => RegistryStatus::Broken { reason: format!("{url} answered: {text}") },
            },
            Err(LauncherError::Offline { .. }) => RegistryStatus::Offline,
            Err(error) => RegistryStatus::Broken { reason: error.to_string() },
        };
        self.status = outcome.clone();
        outcome
    }

    pub fn page(&mut self, api: u32, page: u32) -> Result<Cached<IndexPage>, LauncherError> {
        let key = (api, page);
        if let Some(position) = self.pages.iter().position(|(held, _)| *held == key)
            && let Some(entry) = self.pages.remove(position)
        {
            let value = entry.1.clone();
            self.pages.push_back(entry);
            return Ok(value);
        }
        let relative = format!("index/api-{api}/{page:04}.json");
        let url = self.registry.url(&relative);
        let raw = self.load(&relative, FileKind::IndexPage)?;
        let parsed: IndexPage = Self::parse(&url, &raw.bytes)?;
        if parsed.schema != Self::SCHEMA {
            return Err(LauncherError::BadSchema { url, found: parsed.schema });
        }
        if parsed.api_version != api || parsed.page != page {
            return Err(LauncherError::Http { url, text: "the index page does not match its path".to_string() });
        }
        self.remember(&relative, &raw);
        let cached = Cached { value: parsed, stale: raw.stale };
        self.pages.push_back((key, cached.clone()));
        while self.pages.len() > Self::PAGES_IN_MEMORY {
            self.pages.pop_front();
        }
        Ok(cached)
    }

    pub fn names(&mut self) -> Result<Cached<NamesFile>, LauncherError> {
        if let Some(held) = &self.names {
            return Ok(held.clone());
        }
        if self.root.is_none() {
            self.refresh();
        }
        let Some(root) = self.root.clone() else {
            return Err(LauncherError::Offline { url: self.registry.url(Self::ROOT_PATH) });
        };
        let relative = root.names.path.clone();
        let url = self.registry.url(&relative);
        let raw = self.load(&relative, FileKind::Names)?;
        if !raw.stale {
            let found = HashText::sha256(&raw.bytes);
            if found != root.names.sha256 || raw.bytes.len() as u64 != root.names.size {
                return Err(LauncherError::ChecksumMismatch { file: relative, expected: root.names.sha256, found });
            }
        }
        let parsed: NamesFile = Self::parse(&url, &raw.bytes)?;
        if parsed.schema != Self::SCHEMA {
            return Err(LauncherError::BadSchema { url, found: parsed.schema });
        }
        self.remember(&relative, &raw);
        let cached = Cached { value: parsed, stale: raw.stale };
        self.names = Some(cached.clone());
        Ok(cached)
    }

    // always asks the server, with the cached etag; a registry that is down answers from the cache
    pub fn document(&mut self, id: &ModId) -> Result<Cached<ModDocument>, LauncherError> {
        let relative = id.document_path();
        let url = self.registry.url(&relative);
        let raw = self.load(&relative, FileKind::ModDocument)?;
        let parsed: ModDocument = Self::parse(&url, &raw.bytes)?;
        if parsed.schema != Self::SCHEMA {
            return Err(LauncherError::BadSchema { url, found: parsed.schema });
        }
        if parsed.id != id.as_str() {
            return Err(LauncherError::Http {
                url,
                text: format!("the document is for {}, not {}", parsed.id, id.as_str()),
            });
        }
        self.remember(&relative, &raw);
        Ok(Cached { value: parsed, stale: raw.stale })
    }

    // the icon path of a card or listing: below `icons/`, at most 64 KB
    pub fn icon(&mut self, relative: &str) -> Result<Cached<Vec<u8>>, LauncherError> {
        if !relative.starts_with("icons/") {
            return Err(LauncherError::BadModId(relative.to_string()));
        }
        let raw = self.load(relative, FileKind::Icon)?;
        self.remember(relative, &raw);
        Ok(Cached { value: raw.bytes, stale: raw.stale })
    }

    fn accept_root(&mut self, url: &str, raw: RawFile) -> RegistryStatus {
        let value: serde_json::Value = match serde_json::from_slice(&raw.bytes) {
            Ok(value) => value,
            Err(error) => return RegistryStatus::Broken { reason: format!("{url} is not JSON: {error}") },
        };
        let schema = value.get("schema").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
        if schema != Self::SCHEMA {
            return RegistryStatus::Broken {
                reason: LauncherError::BadSchema { url: url.to_string(), found: schema }.to_string(),
            };
        }
        let root: RegistryRoot = match serde_json::from_value(value) {
            Ok(root) => root,
            Err(error) => return RegistryStatus::Broken { reason: format!("{url} is not a registry root: {error}") },
        };
        if self.root.as_ref() != Some(&root) {
            self.pages.clear();
            self.names = None;
        }
        self.root = Some(root);
        self.remember(Self::ROOT_PATH, &raw);
        if raw.stale {
            RegistryStatus::Stale { reason: format!("{url} cannot be reached; showing the saved copy") }
        } else {
            RegistryStatus::Ok
        }
    }

    // a conditional GET with the cached etag; a registry that cannot be reached answers from the disk copy
    fn load(&self, relative: &str, kind: FileKind) -> Result<RawFile, LauncherError> {
        RegistryCache::safe_relative(relative)?;
        let cached = self.cache.read(&self.registry, relative);
        let etag = cached.as_ref().and_then(|(_, etag)| etag.clone());
        let url = self.registry.url(relative);
        match self.http.get(&url, etag.as_deref(), kind.limit()) {
            Ok(Fetched::Body { bytes, etag }) => Ok(RawFile { bytes, etag, stale: false, fresh: true }),
            Ok(Fetched::NotModified) => match cached {
                Some((bytes, etag)) => Ok(RawFile { bytes, etag, stale: false, fresh: false }),
                None => Err(LauncherError::Http { url, text: "not modified, but nothing is cached".to_string() }),
            },
            Err(LauncherError::Offline { url }) => match cached {
                Some((bytes, etag)) => Ok(RawFile { bytes, etag, stale: true, fresh: false }),
                None => Err(LauncherError::Offline { url }),
            },
            Err(error) => Err(error),
        }
    }

    fn remember(&self, relative: &str, raw: &RawFile) {
        if !raw.fresh {
            return;
        }
        if let Err(error) = self.cache.write(&self.registry, relative, &raw.bytes, raw.etag.as_deref()) {
            curia::warn!("a registry file was not cached", { file: relative, error: error.to_string() });
        }
    }

    fn parse<T: DeserializeOwned>(url: &str, bytes: &[u8]) -> Result<T, LauncherError> {
        serde_json::from_slice(bytes)
            .map_err(|error| LauncherError::Http { url: url.to_string(), text: format!("not valid: {error}") })
    }
}
